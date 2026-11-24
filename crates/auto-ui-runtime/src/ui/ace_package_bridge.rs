//! PLAN-980 ④-A 首片 #2b：**AGE 包闭包的桌面桥引擎**。
//!
//! 组件面引擎（`AceStateEngine`）只吃单源；本引擎把"编译单元集合 + 入口"作为
//! 输入。普通入口挂载经过校验的 AV 程序，并以当前模型处理单个事件。
//! 场景观察和有状态子组件仍走原包场景链，两条入口都由 AV 求值。
//!
//! 首版边界（具名）：只观察 `int/i32` model 字段（`observe_age_package` 的
//! 状态面即该域）；包面场景**无 initials 面**，非空 props 种子具名拒；
//! Payload events are directed to the entry component and checked by the package front door.
use ace_av::age_component::{ComponentScenario, InitialValue, LegacyWidgetAlias, ScenarioOp};
use ace_av::age_package::{ExecLimits, WidgetViewItem};
use ace_core::atom_text::Span;
use ace_core::source::age_component::ComponentNode;
use ace_core::source::age_package::{
    compile_age_package_with_scenario, widget_timer_interval_ms, widget_view_fn_name_at,
    AgePackageCompilation, ImageSurfaceEventPayload, TODO_SERVICE_BINDING,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

use crate::ui::ace_state_bridge::{initial_of, AceStateValue, DispatchOutcome, DropReason};
use crate::back_proxy::{FileBackendSession, SessionSpec};
use crate::ui::iced::image_surface_input::ImageSurfaceCarry;
use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, Mutex}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

mod frame;
mod live;
pub(crate) mod file_directory;
pub(crate) mod file_mutation;
pub(crate) mod file_open;
pub(crate) mod preferences;
mod todo_service;
pub use frame::{PackageAction, PackageFrame};
pub(crate) use frame::{PackageFocusNotice, PackageQueuedAction};
use auto_lang_core::file_session::{FileSession, FileWritePermissions};
use file_directory::FileDirectoryBinding;
use file_mutation::{FileMutationBinding, MutationRequest};
use file_open::FileOpenBinding;
use live::{LiveEvent, LivePackage, MountedWidget};
use std::path::{Path, PathBuf};
use ase_m0::todo_service::{RealmId, ServiceBinding};
use todo_service::TodoTab;

#[derive(Debug)]
pub(crate) enum PackageEventInput {
    Activation,
    Text(String),
    Pointer { x: f32, y: f32 },
    ImageSurface(ImageSurfaceCarry),
}

impl PackageEventInput {
    fn surface_payload(&self) -> Option<ImageSurfaceEventPayload> {
        match self {
            Self::ImageSurface(ImageSurfaceCarry::Wheel { .. }) => Some(ImageSurfaceEventPayload::Wheel),
            Self::ImageSurface(ImageSurfaceCarry::Pan { .. }) => Some(ImageSurfaceEventPayload::Pan),
            _ => None,
        }
    }

    fn matches_surface(&self, action: &PackageAction) -> bool {
        let required = action.source.image_surface_payload;
        match self {
            Self::ImageSurface(_) => required.is_some() && self.surface_payload() == required,
            _ => required.is_none(),
        }
    }

    fn arguments(self, action: &PackageAction) -> Result<Vec<AceStateValue>, String> {
        let refused = || String::from("ace.desktop.widget-input: invalid host event payload");
        if !self.matches_surface(action) {
            return Err(refused());
        }
        let motion = action.source.key.eq_ignore_ascii_case("onmousemove");
        Ok(match self {
            Self::Pointer { x, y } if motion && x.is_finite() && y.is_finite() => {
                vec![AceStateValue::F32(x), AceStateValue::F32(y)]
            }
            Self::Text(text) if action.input_value => vec![AceStateValue::Str(text)],
            Self::Activation if !motion && !action.input_value => {
                action.source.args.iter().map(AceStateValue::from).collect()
            }
            Self::ImageSurface(ImageSurfaceCarry::Wheel { delta_y, .. }) if delta_y.is_finite() => {
                vec![AceStateValue::F32(delta_y)]
            }
            Self::ImageSurface(ImageSurfaceCarry::Pan { dx, dy, phase }) if dx.is_finite() && dy.is_finite() => {
                vec![AceStateValue::F32(dx), AceStateValue::F32(dy), AceStateValue::Str(phase.as_str().into())]
            }
            _ => return Err(refused()),
        })
    }
}

fn frame_limits() -> ExecLimits {
    // A checked frame can replay and materialize a full file listing or tree.
    // Give each entry time to use the existing instruction budget on desktop.
    ExecLimits {
        max_steps: 6_000_000,
        wall: Duration::from_secs(4),
        ..Default::default()
    }
}

/// 一个实例槽（keyed identity + 该实例的定向 op 历史）。
#[derive(Clone)]
struct Slot {
    identity: String,
    live: bool,
    generation: u64,
    ops: Vec<ScenarioOp>,
    frame: Option<Arc<PackageFrame>>,
    widget: Option<Arc<Mutex<MountedWidget>>>,
    initialized: BTreeSet<usize>,
    timer_due: Vec<Instant>,
    child_timer_due: BTreeMap<frame::ChildTimerKey, Instant>,
    file_session: Option<Arc<FileSession>>,
    file_backend: Option<Arc<FileBackendSession>>,
}

impl Slot {
    fn publish_frame(&mut self, frame: PackageFrame, now: Instant) {
        let mut due = BTreeMap::new();
        for timer in &frame.child_timers {
            let key = &timer.key;
            let deadline = self.child_timer_due.get(key).copied().unwrap_or_else(|| {
                now + Duration::from_millis(u64::from(key.every_ms))
            });
            due.insert(key.clone(), deadline);
        }
        self.child_timer_due = due;
        self.frame = Some(Arc::new(frame));
    }
}

struct PackageTimer {
    interval: Duration,
    message: String,
}

struct PackageHttpWork {
    generation: u64,
    message: String,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<PackageFrame, String>>>,
}

impl Drop for PackageHttpWork {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
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
    live_package: Option<Arc<LivePackage>>,
    http_work: BTreeMap<usize, PackageHttpWork>,
    timers: Vec<PackageTimer>,
    file_directory: Option<FileDirectoryBinding>,
    file_mutation: Option<FileMutationBinding>,
    file_open: Option<FileOpenBinding>,
    file_backend: Option<(SessionSpec, PathBuf)>,
    preferences: Vec<auto_ui_dialect::preferences::PreferenceBinding>,
    todo_reply: Option<String>,
    todo_service: Option<(ServiceBinding, RealmId)>,
    todo_tabs: BTreeMap<String, TodoTab>,
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
        let (entry_node, component) = compiled
            .description
            .components
            .iter()
            .enumerate()
            .find(|(_, component)| component.module == entry)
            .ok_or_else(|| format!("entry module `{entry}` declares no component"))?;
        let has_entry_view = compiled.symbol_of_in(entry, &widget_view_fn_name_at(entry_node, 0)).is_some();
        if !has_entry_view && component.members.iter().any(|row| row.role == "timer") {
            return Err("ace.desktop.timer-view: timers require an ordinary checked entry view".into());
        }
        let timers: Vec<_> = component.members.iter().filter(|row| row.role == "timer")
            .map(|row| PackageTimer {
                interval: Duration::from_millis(
                    row.num.and_then(widget_timer_interval_ms).expect("checked timer interval").into(),
                ),
                message: row.name.clone(),
            })
            .collect();
        let checked_views = compiled.first_widget_ref().is_some() || has_entry_view;
        let file_directory = FileDirectoryBinding::from_package(&compiled, component)?;
        let file_mutation = FileMutationBinding::from_component(component)?;
        let file_open = FileOpenBinding::from_component(component)?;
        let preferences = preferences::checked_bindings(component)?;
        let todo_reply = component.members.iter()
            .find(|row| row.role == "bind" && row.name == TODO_SERVICE_BINDING)
            .and_then(|row| row.text.clone());
        if !preferences.is_empty() && !has_entry_view {
            return Err("ace.desktop.storage-binding: storage requires a checked entry view".into());
        }
        if (file_directory.is_some() || file_mutation.is_some() || file_open.is_some()) && !has_entry_view {
            return Err("ace.desktop.file-binding: file requests require a checked entry view".into());
        }
        let live_package = if has_entry_view {
            LivePackage::prepare(&units, entry, alias, &compiled, component)?.map(Arc::new)
        } else {
            None
        };
        if todo_reply.is_some() && live_package.is_none() {
            return Err("ace.desktop.todo-binding: a checked live entry view is required".into());
        }
        let startup_handlers = if checked_views && live_package.is_none() {
            Self::initial_widget_handlers(&compiled, entry_node)?
        } else {
            BTreeSet::new()
        };
        let engine = Self {
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
            live_package,
            http_work: BTreeMap::new(),
            timers,
            file_directory,
            file_mutation,
            file_open,
            file_backend: None,
            preferences,
            todo_reply,
            todo_service: None,
            todo_tabs: BTreeMap::new(),
        };
        // Verify that both reply constructors match the package before a host can
        // authorize any access. This compilation never runs the filesystem.
        if let Some(binding) = &engine.file_directory {
            engine.compile(1, binding.check_replies(entry_node))?;
        }
        let node = compiled.entry_node().ok_or_else(|| format!("entry module `{entry}` declares no component"))?;
        Ok((engine, node.clone()))
    }

    pub(crate) fn has_checked_view(&self) -> bool {
        self.checked_views
    }

    pub(super) fn set_file_backend(&mut self, spec: SessionSpec, source_dir: &Path) {
        self.file_backend = Some((spec, source_dir.to_path_buf()));
    }

    pub(crate) fn install_todo_service(&mut self, binding: ServiceBinding, realm: RealmId) -> Result<(), String> {
        if self.todo_reply.is_none() || self.slots.iter().any(|slot| slot.live) {
            return Err("ace.desktop.todo-binding: install a service on a checked, unmounted Todo package".into());
        }
        self.todo_service = Some((binding, realm));
        Ok(())
    }

    pub(crate) fn poll_todo_service(&mut self, identity: &str) -> Result<Option<(String, Vec<(String, AceStateValue)>)>, String> {
        let Some(index) = self.slot(identity).filter(|&index| self.slots[index].live) else {
            return Ok(None);
        };
        let Some(tab) = self.todo_tabs.get_mut(identity) else {
            return Ok(None);
        };
        let slot = &mut self.slots[index];
        let frame = slot.frame.as_ref().ok_or("Todo service has no checked frame")?;
        let payload = match tab.poll() {
            Some(payload) => Some(payload),
            None => {
                use todo_service::{DONE_FIELD, ID_FIELD, OPERATION_FIELD, REQUEST_FIELD, TEXT_FIELD};
                let state = |name| frame.states.iter().find(|(field, _)| {
                    field.strip_prefix("st_").unwrap_or(field) == name
                }).map(|(_, value)| value);
                let (Some(AceStateValue::Int(sequence)), Some(AceStateValue::Str(operation)),
                    Some(AceStateValue::Int(id)), Some(AceStateValue::Str(text)),
                    Some(AceStateValue::Bool(done))) =
                    (state(REQUEST_FIELD), state(OPERATION_FIELD), state(ID_FIELD), state(TEXT_FIELD), state(DONE_FIELD))
                else {
                    return Err("ace.desktop.todo-binding: invalid checked Todo request fields".into());
                };
                tab.observe(*sequence, operation, *id, text, *done)
            }
        };
        let Some(payload) = payload else { return Ok(None) };
        let live = self.live_package.as_ref().ok_or("Todo service has no live package")?;
        let mount = slot.widget.as_deref().ok_or("Todo service mount was released")?;
        let message = self.todo_reply.as_deref().ok_or("Todo service has no checked reply")?;
        let next = live.todo_reply(mount, frame.revision + 1, message, payload)?;
        let states = next.states.clone();
        slot.frame = Some(Arc::new(next));
        Ok(Some((message.to_owned(), states)))
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
        let file_backend = self.file_backend.as_ref().map(|(spec, source)| {
            crate::back_proxy::native_files::open(spec, source).map(Arc::new)
        }).transpose()?;
        let file_session = file_backend.as_ref().map(|backend| backend.file_access())
            .or_else(|| {
                (self.file_directory.is_some() || self.file_mutation.is_some() || self.file_open.is_some())
                    .then(|| Arc::new(FileSession::default()))
            });
        let todo = if self.todo_reply.is_some() {
            let (binding, realm) = self.todo_service.as_ref()
                .ok_or("ace.desktop.todo-binding: host has not granted the local Todo service")?;
            Some(TodoTab::open(binding, *realm).map_err(|error| format!("Todo lease: {error:?}"))?)
        } else {
            None
        };
        let mut slot = Slot {
            identity: identity.to_string(), live: true, generation,
            ops: Vec::new(), frame: None, initialized: BTreeSet::new(), timer_due: Vec::new(),
                child_timer_due: BTreeMap::new(),
            widget: None,
            file_session,
            file_backend,
        };
        if let Some(live) = &self.live_package {
            let mount = live.mount(generation, &self.preferences)?;
            let frame = live.update(
                &mount, 1, None, slot.file_session.as_deref(), self.file_directory.as_ref(),
                self.file_mutation.as_ref(), self.file_open.as_ref(), None,
            )?;
            slot.publish_frame(frame, Instant::now());
            slot.widget = Some(mount);
        } else if self.checked_views {
            slot.initialized = self.startup_handlers.clone();
            slot.ops.extend(self.startup_handlers.iter().copied().map(Self::init_op));
            for (message, value) in preferences::restore(&self.preferences) {
                slot.ops.push(ScenarioOp::ForNode {
                    node: self.entry_node,
                    op: Box::new(ScenarioOp::send_args_from_values(
                        0, message, &[InitialValue::Str(value)], self.event_anchor,
                    )),
                });
            }
            self.refresh_frame(&mut slot)?;
            self.resolve_frame_requests(&mut slot)?;
        }
        self.next_generation += 1;
        if let Some(i) = self.slot(identity) {
            self.slots[i] = slot;
        } else {
            self.slots.push(slot);
        }
        if let Some(tab) = todo {
            self.todo_tabs.insert(identity.to_owned(), tab);
        }
        Ok(generation)
    }

    pub(crate) fn unmount(&mut self, identity: &str) {
        self.todo_tabs.remove(identity);
        if let Some(i) = self.slot(identity) {
            self.http_work.remove(&i);
            let slot = &mut self.slots[i];
            slot.live = false;
            slot.frame = None;
            slot.widget = None;
            slot.ops.clear();
            slot.initialized.clear();
            slot.timer_due.clear();
            slot.child_timer_due.clear();
            let access = slot.file_session.take();
            if let Some(backend) = slot.file_backend.take() {
                backend.close();
            } else if let Some(session) = access {
                session.close();
            }
        }
    }

    pub(crate) fn grant_file_read(&self, identity: &str, root: &Path) -> Result<(), String> {
        self.grant_file_access(identity, root, FileWritePermissions::default())
    }

    pub(crate) fn grant_file_access(&self, identity: &str, root: &Path, writes: FileWritePermissions) -> Result<(), String> {
        self.file_session(identity)?.grant(root, writes).map_err(|error| error.to_string())
    }

    pub(crate) fn revoke_file_read(&self, identity: &str) -> Result<(), String> {
        self.file_session(identity)?.revoke();
        Ok(())
    }

    fn file_session(&self, identity: &str) -> Result<&FileSession, String> {
        self.slot(identity)
            .and_then(|index| self.slots.get(index))
            .filter(|slot| slot.live)
            .and_then(|slot| slot.file_session.as_deref())
            .ok_or_else(|| "ace.desktop.file-session: no live file binding for the instance".into())
    }

    pub(crate) fn start_timers(&mut self, now: Instant) {
        for slot in self.slots.iter_mut().filter(|slot| slot.live) {
            slot.timer_due = self.timers.iter().map(|timer| now + timer.interval).collect();
            for (key, due) in &mut slot.child_timer_due {
                *due = now + Duration::from_millis(u64::from(key.every_ms));
            }
        }
    }

    pub(crate) fn dispatch_due_timer(
        &mut self,
        now: Instant,
    ) -> Option<(usize, String, Result<DispatchOutcome, String>)> {
        let mut due = None;
        for (instance, slot) in self.slots.iter_mut().enumerate() {
            if self.http_work.contains_key(&instance) {
                continue;
            }
            let Some(frame) = &slot.frame else { continue };
            for (timer, time) in slot.timer_due.iter_mut().enumerate() {
                if *time > now {
                    continue;
                }
                if !frame.timer_enabled[timer] {
                    *time = now + self.timers[timer].interval;
                    continue;
                }
                if due.is_none_or(|(earliest, _, _)| *time < earliest) {
                    due = Some((*time, instance, timer));
                }
            }
        }
        let child = self.slots.iter().enumerate()
            .filter_map(|(index, slot)| {
                if !slot.live || self.http_work.contains_key(&index) {
                    return None;
                }
                let (key, deadline) = slot.child_timer_due.iter()
                    .filter(|(_, due)| **due <= now)
                    .min_by_key(|(_, due)| **due)?;
                let timer = slot.frame.as_ref()?.child_timers.iter().find(|timer| timer.key == *key)?;
                Some((*deadline, index, timer))
            })
            .min_by_key(|(deadline, _, _)| *deadline);
        if let Some((_, index, timer)) =
            child.filter(|(deadline, _, _)| due.is_none_or(|(root, _, _)| *deadline < root))
        {
            let timer = timer.clone();
            self.slots[index].child_timer_due.insert(
                timer.key.clone(),
                now + Duration::from_millis(u64::from(timer.key.every_ms)),
            );
            let anchor = Span { start: timer.anchor, end: timer.anchor };
            let message = timer.key.message;
            let result = self.dispatch_checked(index, timer.node, &timer.instance, &message, &[], anchor);
            return Some((index, message, result));
        }
        let (_, instance, index) = due?;
        let timer = &self.timers[index];
        self.slots[instance].timer_due[index] = now + timer.interval;
        let message = timer.message.clone();
        let result = self.dispatch_checked(instance, self.entry_node, &[], &message, &[], self.event_anchor);
        Some((instance, message, result))
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
            //
            // PLAN-118（T-03 真窗）：预 Init 求值可能读尚未种下的列表元素
            // （031-paint 的视图叶 `.px[42]`，而 `.px` 由 `.Init` 填 256 项）——
            // 求值失败不得让装载失败：退回「入口 + 全部站点」的可见集，
            // Init 补齐状态后按真实帧重算。
            match ace_av::age_package::evaluate_package_entry_view(compiled, 0, &frame_limits()) {
                Ok(items) => {
                    let mut pending: Vec<_> = items.iter().collect();
                    while let Some(item) = pending.pop() {
                        let WidgetViewItem::Element(element) = item else { continue };
                        if let Some(site) = frame::child_site(&compiled.description, &compiled.widget_refs, entry, element) {
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
                Err(_) => {
                    let sites = compiled.widget_refs.get(&(root.module.clone(), root.name.clone()));
                    visible.extend(sites.into_iter().flatten().filter_map(|site| {
                        compiled.description.components.iter().position(|node| node.module == site.module && node.name == site.node)
                    }));
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
        self.refresh_frame_from(slot, None)
    }

    fn refresh_frame_from(&self, slot: &mut Slot, mut first: Option<AgePackageCompilation>) -> Result<(), String> {
        let revision = slot.frame.as_ref().map_or(1, |frame| frame.revision + 1);
        loop {
            let compiled = match first.take() {
                Some(compiled) => compiled,
                None => self.compile(1, slot.ops.clone())?,
            };
            let frame = frame::render(&compiled, self.entry_node, revision)?;
            if !Self::init_visible(&compiled, frame.mounted.clone(), slot) {
                slot.publish_frame(frame, Instant::now());
                return Ok(());
            }
        }
    }

    fn resolve_directory_request(&self, trial: &mut Slot) -> Result<(), String> {
        let Some(binding) = &self.file_directory else { return Ok(()) };
        let frame = trial.frame.as_ref().expect("checked frame before file effects");
        let Some(path) = binding.pending_path(&frame.states)?.map(str::to_owned) else { return Ok(()) };
        self.complete_directory_request(trial, binding, &path)
    }

    fn complete_directory_request(&self, trial: &mut Slot, binding: &FileDirectoryBinding, path: &str)
        -> Result<(), String>
    {
        let access = trial.file_session.as_ref().expect("file binding owns a session");
        // Only this event commit executes I/O. Replays receive its typed reply
        // from the scenario and never ask the filesystem again.
        let listing = FileDirectoryBinding::list(access, path);
        trial.ops.push(binding.reply(self.entry_node, listing));
        self.refresh_frame(trial)?;
        let frame = trial.frame.as_ref().expect("checked reply frame");
        if binding.pending_path(&frame.states)?.is_some() {
            return Err("ace.desktop.file-reply: directory reply must clear the pending request".into());
        }
        Ok(())
    }

    fn refresh_after_event(&self, trial: &mut Slot) -> Result<(), String> {
        let mut fields = Vec::new();
        if self.file_directory.is_some() { fields.extend_from_slice(file_directory::REQUEST_FIELDS); }
        if self.file_mutation.is_some() { fields.extend_from_slice(file_mutation::REQUEST_FIELDS); }
        if self.file_open.is_some() { fields.extend_from_slice(file_open::REQUEST_FIELDS); }
        if fields.is_empty() { return self.refresh_frame(trial); }
        let compiled = self.compile(1, trial.ops.clone())?;
        // Inspect the checked intent before rendering any intermediate view.
        let request: Vec<_> = ace_av::age_package::observe_package_scalar_fields(
            &compiled, self.entry_node, 0, Some(&fields), &frame_limits(),
        )?.into_iter().map(|(name, value)| (name, AceStateValue::from(&value))).collect();
        if let Some(binding) = &self.file_mutation {
            if let Some(action) = binding.pending_request(&request)? {
                return self.complete_mutation_request(trial, binding, action);
            }
        }
        if let Some(binding) = &self.file_directory {
            if let Some(path) = binding.pending_path(&request)? {
                return self.complete_directory_request(trial, binding, path);
            }
        }
        if let Some(binding) = &self.file_open {
            if let Some(path) = binding.pending_path(&request)? {
                return self.complete_open_request(trial, binding, path);
            }
        }
        self.refresh_frame_from(trial, Some(compiled))?;
        self.resolve_frame_requests(trial)
    }

    fn resolve_frame_requests(&self, trial: &mut Slot) -> Result<(), String> {
        if let Some(binding) = &self.file_mutation {
            let frame = trial.frame.as_ref().expect("checked frame before file effects").clone();
            if let Some(action) = binding.pending_request(&frame.states)? {
                return self.complete_mutation_request(trial, binding, action);
            }
        }
        if let Some(binding) = &self.file_open {
            let frame = trial.frame.as_ref().expect("checked frame before file effects").clone();
            if let Some(path) = binding.pending_path(&frame.states)? {
                return self.complete_open_request(trial, binding, path);
            }
        }
        self.resolve_directory_request(trial)
    }

    fn complete_open_request(
        &self, trial: &mut Slot, binding: &FileOpenBinding, path: &str,
    ) -> Result<(), String> {
        let prepare = |status| {
            let mut reply = trial.clone();
            reply.ops.push(binding.reply(self.entry_node, status));
            self.refresh_frame(&mut reply)?;
            let frame = reply.frame.as_ref().expect("checked file open reply frame");
            if binding.pending_path(&frame.states)?.is_some() {
                return Err("ace.desktop.file-reply: open reply must clear the pending request".into());
            }
            Ok::<_, String>(reply)
        };
        if !cfg!(windows) {
            *trial = prepare(file_open::UNAVAILABLE)?;
            return Ok(());
        }
        let opened = prepare(file_open::OPENED)?;
        let refused = prepare(file_open::DENIED)?;
        let access = trial.file_session.as_ref().expect("file binding owns a session");
        *trial = if file_open::open_document(access, path) { opened } else { refused };
        Ok(())
    }

    fn complete_mutation_request(
        &self, trial: &mut Slot, binding: &FileMutationBinding, action: MutationRequest<'_>,
    ) -> Result<(), String> {
        // Prepare both replies before touching the disk. A failed reply or
        // invalid view therefore cannot leave an uncommitted write behind.
        let prepare = |ok| {
            let mut reply = trial.clone();
            reply.ops.push(binding.reply(self.entry_node, ok));
            self.refresh_frame(&mut reply)?;
            let frame = reply.frame.as_ref().expect("checked mutation reply frame");
            if binding.pending_request(&frame.states)?.is_some() {
                return Err("ace.desktop.file-reply: mutation reply must clear the pending request".into());
            }
            Ok::<_, String>(reply)
        };
        let accepted = prepare(true)?;
        let refused = prepare(false)?;
        let access = trial.file_session.as_ref().expect("file binding owns a session");
        *trial = if action.execute(access) { accepted } else { refused };

        // The write has committed. If rendering a follow-up read fails, keep
        // the checked mutation reply so the next dispatch cannot repeat I/O.
        let mut refreshed = trial.clone();
        match self.resolve_directory_request(&mut refreshed) {
            Ok(()) => *trial = refreshed,
            Err(error) => log::warn!("ace.desktop.file-reply: mutation refresh failed: {error}"),
        }
        Ok(())
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
        if self.slots[inst].widget.is_some() {
            return self.slots[inst].frame.as_ref().map(|frame| {
                fn collect(items: &[WidgetViewItem], output: &mut Vec<String>) {
                    for item in items {
                        match item {
                            WidgetViewItem::Text(text) => output.push(text.clone()),
                            WidgetViewItem::Element(element) => collect(&element.children, output),
                            WidgetViewItem::Timer(_) => {}
                        }
                    }
                }
                let mut texts = Vec::new();
                collect(&frame.view, &mut texts);
                PackageSnapshot { states: frame.states.clone(), texts }
            });
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
        input: PackageEventInput,
    ) -> Result<(String, DispatchOutcome), String> {
        let frame = self.package_frame(identity).ok_or("checked view is not mounted")?;
        if revision != frame.revision {
            return Err("ace.desktop.widget-event-stale: view revision has changed".into());
        }
        let action = frame.enabled_action(target).ok_or("unknown or disabled checked view event target")?;
        let args = input.arguments(action)?;
        let index = self.slot(identity).expect("frame has a mounted slot");
        let outcome = self.dispatch_checked(index, action.node, &action.instance, &action.message, &args, action.anchor)?;
        Ok((action.message.clone(), outcome))
    }

    fn dispatch_checked(&mut self, index: usize, node: usize, instance: &[usize], message: &str, args: &[AceStateValue], anchor: Span)
        -> Result<DispatchOutcome, String>
    {
        if node == self.entry_node && self.preferences.iter().any(|binding| binding.restore == message) {
            return Err("ace.desktop.storage-binding: only the storage host can restore preferences".into());
        }
        if node == self.entry_node && self.file_open.as_ref().is_some_and(|binding| binding.reply_name() == message) {
            return Err("ace.desktop.file-reply: only the file binding can deliver its reply".into());
        }
        if let Some(live) = &self.live_package {
            if live.uses_http() {
                return self.start_http_update(index, LiveEvent { instance, message, args });
            }
            let slot = &mut self.slots[index];
            let mount = slot.widget.as_deref().ok_or("checked widget is not mounted")?;
            let previous = slot.frame.as_ref().ok_or("missing checked widget frame")?;
            let revision = previous.revision + 1;
            let frame = live.update(
                mount, revision, Some(LiveEvent { instance, message, args }), slot.file_session.as_deref(),
                self.file_directory.as_ref(), self.file_mutation.as_ref(), self.file_open.as_ref(), None,
            )?;
            let states = frame.states.clone();
            preferences::save_changes(&self.preferences, &previous.states, &states);
            slot.publish_frame(frame, Instant::now());
            return Ok(DispatchOutcome::Delivered(states));
        }
        let mut trial = self.slots[index].clone();
        let values: Vec<_> = args.iter().map(initial_of).collect();
        trial.ops.push(ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::send_args_from_values(
                0, message, &values, Span { start: anchor.start, end: anchor.start },
            )),
        });
        self.refresh_after_event(&mut trial)?;
        let states = trial.frame.as_ref().expect("successful render produced a frame").states.clone();
        if let Some(previous) = &self.slots[index].frame {
            preferences::save_changes(&self.preferences, &previous.states, &states);
        }
        self.slots[index] = trial;
        Ok(DispatchOutcome::Delivered(states))
    }

    fn start_http_update(&mut self, index: usize, event: LiveEvent<'_>) -> Result<DispatchOutcome, String> {
        if self.http_work.contains_key(&index) {
            return Ok(DispatchOutcome::Dropped(DropReason::HttpBusy));
        }
        let live = Arc::clone(self.live_package.as_ref().expect("HTTP requires a live package"));
        let slot = &self.slots[index];
        let mount = slot.widget.clone().ok_or("checked widget is not mounted")?;
        let revision = slot.frame.as_ref().ok_or("missing checked widget frame")?.revision + 1;
        let file = slot.file_session.clone();
        let backend = slot.file_backend.clone();
        let directory = self.file_directory.clone();
        let mutation = self.file_mutation.clone();
        let open = self.file_open.clone();
        let instance = event.instance.to_vec();
        let args = event.args.to_vec();
        let message = event.message.to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let worker_message = message.clone();
        let worker = std::thread::Builder::new()
            .name("ace-package-http".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                live.update(
                    &mount, revision,
                    Some(LiveEvent { instance: &instance, message: &worker_message, args: &args }),
                    file.as_deref(), directory.as_ref(), mutation.as_ref(), open.as_ref(), Some((worker_cancel, backend)),
                )
            })
            .map_err(|error| format!("start checked package HTTP worker: {error}"))?;
        self.http_work.insert(index, PackageHttpWork {
            generation: slot.generation,
            message,
            cancel,
            worker: Some(worker),
        });
        Ok(DispatchOutcome::InFlight)
    }

    pub(crate) fn http_pending(&self, identity: &str) -> bool {
        self.slot(identity).is_some_and(|index| self.http_work.contains_key(&index))
    }

    pub(crate) fn poll_http_update(
        &mut self,
    ) -> Option<(usize, String, Result<Vec<(String, AceStateValue)>, String>)> {
        let index = self.http_work.iter()
            .find(|(_, work)| work.worker.as_ref().is_some_and(|worker| worker.is_finished()))
            .map(|(&index, _)| index)?;
        let mut work = self.http_work.remove(&index).expect("finished worker");
        let slot = &mut self.slots[index];
        if !slot.live || slot.generation != work.generation {
            return None;
        }
        let result = work.worker.take().expect("finished worker").join()
            .map_err(|_| "checked package HTTP worker panicked".to_string())
            .and_then(|frame| frame)
            .map(|frame| {
                let previous = slot.frame.as_ref().expect("live HTTP worker has a published frame");
                preferences::save_changes(&self.preferences, &previous.states, &frame.states);
                let states = frame.states.clone();
                slot.publish_frame(frame, Instant::now());
                states
            });
        Some((index, std::mem::take(&mut work.message), result))
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
        self.dispatch_checked(inst, self.entry_node, &[], msg, args.unwrap_or_default(), self.event_anchor)
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
    compile_age_package_with_scenario(units, entry, alias, &caps, &scenario)
        .map_err(|diagnostics| package_diagnostic(&diagnostics))
}

fn package_diagnostic(diagnostics: &[ace_core::atom_text::Diagnostic]) -> String {
    diagnostics
        .first()
        .map(|diagnostic| format!("{}: {} @{:?}", diagnostic.code, diagnostic.message, diagnostic.span))
        .unwrap_or_else(|| "package scene refused".to_string())
}
