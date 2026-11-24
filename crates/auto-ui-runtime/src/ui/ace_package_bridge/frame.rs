use std::collections::{BTreeMap, BTreeSet};

use ace_av::age_package::{WidgetElement, WidgetEvent, WidgetViewItem};
use ace_av::age_package::live_session::WidgetFocusRequest;
use ace_core::source::age_package::{
    AgePackageCompilation, AgePackageDescription, AgeWidgetProperty, AgeWidgetRef, AgeWidgetRefs,
};

use crate::ui::ace_vue_script::channel_pascal;

use super::{AceStateValue, PackageEventInput};

/// A rendered package and the events offered by that exact revision.
#[derive(Debug)]
pub struct PackageFrame {
    pub revision: u64,
    pub view: Vec<WidgetViewItem>,
    pub(super) states: Vec<(String, AceStateValue)>,
    pub(super) mounted: BTreeSet<usize>,
    pub(super) timer_enabled: Vec<bool>,
    pub(super) focus_requests: Vec<WidgetFocusRequest>,
    pub(super) actions: Vec<PackageAction>,
    pub(super) child_timers: Vec<ChildTimer>,
    pointers: BTreeMap<Vec<usize>, PointerEvents>,
    image_surfaces: BTreeMap<Vec<usize>, ImageSurfaceEvents>,
    clicks: BTreeMap<Vec<usize>, usize>,
    /// PLAN-118：canvas onhit 目标（tap 命中 id 经文本事件通道回程序）。
    hits: BTreeMap<Vec<usize>, usize>,
    /// PLAN-118：canvas 笔事件目标（onpenstart/onpenmove/onpenend）。
    pens: BTreeMap<Vec<usize>, (Option<usize>, Option<usize>, Option<usize>)>,
    /// PLAN-118：canvas 场景数据（五平行列表解析后的画布内容，按元素路径）。
    scenes: BTreeMap<Vec<usize>, crate::ui::view::CanvasScene>,
    links: BTreeMap<Vec<usize>, usize>,
    destinations: Vec<String>,
    widgets: BTreeSet<Vec<usize>>,
    controls: BTreeMap<Vec<usize>, PackageControl>,
    focus_targets: BTreeMap<(Vec<usize>, String), Option<String>>,
    focus_actions: BTreeMap<String, [Option<usize>; 2]>,
    queued_actions: BTreeMap<String, Option<usize>>,
}

#[derive(Debug, Default)]
pub(crate) struct PackageControl {
    pub focus: Option<String>,
    pub keys: Vec<(String, usize)>,
    pub context_menu: Option<usize>,
    pub double_click: Option<usize>,
    pub input: Option<usize>,
    pub submit: Option<usize>,
    pub on_focus: bool,
    pub on_blur: bool,
}

#[derive(Debug)]
pub(crate) struct PackageFocusNotice {
    target: String,
    gained: bool,
    source: WidgetEvent,
    node: usize,
    message: String,
}

#[derive(Debug)]
pub(crate) struct PackageQueuedAction {
    control: String,
    node: usize,
    instance: Vec<usize>,
    message: String,
    source: WidgetEvent,
}

#[derive(Debug)]
pub struct PackageAction {
    pub source: WidgetEvent,
    pub(super) node: usize,
    pub(super) instance: Vec<usize>,
    pub(super) message: String,
    pub(super) anchor: ace_core::atom_text::Span,
    pub(super) input_value: bool,
    disabled: bool,
    queued_control: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ChildTimerOwner {
    Scenario { path: Vec<usize>, node: usize, instance: Vec<usize> },
    Live(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ChildTimerKey {
    pub owner: ChildTimerOwner,
    pub message: String,
    pub every_ms: u32,
}

#[derive(Debug, Clone)]
pub(super) struct ChildTimer {
    pub key: ChildTimerKey,
    pub node: usize,
    pub instance: Vec<usize>,
    pub anchor: usize,
}

#[derive(Debug, Default)]
pub(crate) struct PointerEvents {
    pub(crate) enter: Option<usize>,
    pub(crate) leave: Option<usize>,
    pub(crate) motion: Option<usize>,
}

#[derive(Debug, Default)]
pub(crate) struct ImageSurfaceEvents {
    pub(crate) loaded: Option<usize>,
    pub(crate) error: Option<usize>,
    pub(crate) wheel: Option<usize>,
    pub(crate) pan: Option<usize>,
    pub(crate) double_click: Option<usize>,
}

impl PackageAction {
    /// PLAN-118：证据面读回动作消息与事件键。
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn key(&self) -> &str {
        &self.source.key
    }
}

impl PackageFrame {
    /// PLAN-118：渲染出的视图项（证据面读回文本叶/元素树）。
    pub fn view(&self) -> &[WidgetViewItem] {
        &self.view
    }

    /// PLAN-118：帧携带的标量状态（事件证据/状态读回）。
    pub fn states(&self) -> &[(String, AceStateValue)] {
        &self.states
    }

    pub fn focus_requests(&self) -> &[WidgetFocusRequest] {
        &self.focus_requests
    }

    pub fn actions(&self) -> &[PackageAction] {
        &self.actions
    }

    /// PLAN-118：canvas onhit 事件目标（渲染层 tap 命中后按此派发）。
    pub(crate) fn hit(&self, path: &[usize]) -> Option<usize> {
        self.hits.get(path).copied()
    }

    /// PLAN-118：脚本 tap 的场景命中——与渲染层同一条 `canvas_hit_test`。
    pub(crate) fn hit_id_at(&self, target: usize, x: f32, y: f32) -> Option<String> {
        for (path, t) in &self.hits {
            if *t != target {
                continue;
            }
            let scene = self.scenes.get(path)?;
            if let Some(id) = crate::ui::view::canvas_hit_test(&scene.nodes, x, y) {
                return Some(id);
            }
        }
        None
    }

    /// PLAN-118：canvas 场景数据读回（五平行列表解析结果）。
    pub fn scene(&self, path: &[usize]) -> Option<&crate::ui::view::CanvasScene> {
        self.scenes.get(path)
    }

    /// PLAN-118：由 live 会话读回的场景数据回填（按元素路径）。
    pub(super) fn set_scene(&mut self, path: Vec<usize>, scene: crate::ui::view::CanvasScene) {
        self.scenes.insert(path, scene);
    }

    /// PLAN-118：canvas 笔事件目标（start/move/end）。
    pub(crate) fn pen(
        &self,
        path: &[usize],
    ) -> Option<(Option<usize>, Option<usize>, Option<usize>)> {
        self.pens.get(path).copied()
    }

    pub(crate) fn click(&self, path: &[usize]) -> Option<usize> {
        self.clicks.get(path).copied()
    }

    pub(crate) fn pointer(&self, path: &[usize]) -> Option<&PointerEvents> {
        self.pointers.get(path)
    }

    pub(crate) fn image_surface(&self, path: &[usize]) -> Option<&ImageSurfaceEvents> {
        self.image_surfaces.get(path)
    }

    pub(crate) fn link(&self, path: &[usize]) -> Option<usize> {
        self.links.get(path).map(|index| self.actions.len() + index)
    }

    pub(crate) fn navigation_target(&self, target: usize) -> Option<&str> {
        let index = target.checked_sub(self.actions.len())?;
        self.destinations.get(index).map(String::as_str)
    }

    pub(crate) fn accepts_target(&self, target: usize, input: &PackageEventInput) -> bool {
        self.enabled_action(target).is_some_and(|action| {
            !action.source.key.eq_ignore_ascii_case("onfocus")
                && !action.source.key.eq_ignore_ascii_case("onblur")
                && input.matches_surface(action)
                && action.source.key.eq_ignore_ascii_case("oninput") == matches!(input, PackageEventInput::Text(_))
                && match input {
                    PackageEventInput::Pen { .. } => {
                        action.source.key.to_ascii_lowercase().starts_with("onpen")
                    }
                    PackageEventInput::Hit(_) => {
                        action.source.key.eq_ignore_ascii_case("onhit")
                    }
                    PackageEventInput::Activation => {
                        let key = action.source.key.to_ascii_lowercase();
                        !key.starts_with("onpen") && key != "onhit"
                    }
                    _ => true,
                }
        }) || (matches!(input, PackageEventInput::Activation) && self.navigation_target(target).is_some())
    }

    pub(super) fn enabled_action(&self, target: usize) -> Option<&PackageAction> {
        self.actions.get(target).filter(|action| !action.disabled)
    }

    pub(crate) fn action_message(&self, target: usize) -> Option<&str> {
        self.enabled_action(target).map(|action| action.message.as_str())
    }

    pub(crate) fn model_values(&self) -> &[(String, AceStateValue)] {
        &self.states
    }

    pub(crate) fn is_widget(&self, path: &[usize]) -> bool {
        self.widgets.contains(path)
    }

    pub(crate) fn control(&self, path: &[usize]) -> Option<&PackageControl> {
        self.controls.get(path)
    }

    pub(crate) fn focus_target(&self) -> Option<&str> {
        self.focus_requests.iter().rev().find_map(|request| {
            let target = focus_target_id(&request.target);
            self.focus_targets.get(&(request.component.clone(), target.to_owned()))?.as_deref()
        })
    }

    fn focus_action(&self, target: &str, gained: bool) -> Option<(usize, &PackageAction)> {
        let action = self.focus_actions.get(target)?[usize::from(gained)]?;
        Some((action, self.enabled_action(action)?))
    }

    pub(crate) fn focus_notice(&self, target: String, gained: bool) -> Option<PackageFocusNotice> {
        let (_, action) = self.focus_action(&target, gained)?;
        Some(PackageFocusNotice {
            target,
            gained,
            source: action.source.clone(),
            node: action.node,
            message: action.message.clone(),
        })
    }

    // A click or the other half of a focus transfer can publish first.
    // Continue only if the same control still offers the exact checked event.
    pub(crate) fn focus_notice_target(&self, notice: &PackageFocusNotice) -> Option<usize> {
        let (target, action) = self.focus_action(&notice.target, notice.gained)?;
        (action.node == notice.node && action.message == notice.message && action.source == notice.source)
            .then_some(target)
    }

    pub(crate) fn queued_action(&self, target: usize) -> Option<PackageQueuedAction> {
        let action = self.enabled_action(target)?;
        Some(PackageQueuedAction {
            control: action.queued_control.clone()?,
            node: action.node,
            instance: action.instance.clone(),
            message: action.message.clone(),
            source: action.source.clone(),
        })
    }

    // Edits can share a frame with Enter, and selection with a double-click.
    // Re-resolve only the same authored control and checked action.
    pub(crate) fn queued_action_target(&self, notice: &PackageQueuedAction) -> Option<usize> {
        let target = (*self.queued_actions.get(&notice.control)?)?;
        let action = self.enabled_action(target)?;
        (action.node == notice.node && action.instance == notice.instance
            && action.message == notice.message && action.source == notice.source)
            .then_some(target)
    }
}

impl PackageFocusNotice {
    pub(crate) fn gained(&self) -> bool {
        self.gained
    }
}

fn focus_target_id(target: &str) -> &str {
    target.strip_prefix('#')
        .or_else(|| target.strip_prefix("[id='")?.strip_suffix("']"))
        .or_else(|| target.strip_prefix("[id=\"")?.strip_suffix("\"]"))
        .unwrap_or(target)
}

pub(super) fn render(
    compiled: &AgePackageCompilation,
    root: usize,
    revision: u64,
) -> Result<PackageFrame, String> {
    let limits = super::frame_limits();
    let view = ace_av::age_package::evaluate_package_view(compiled, 0, &limits)?;
    let states = ace_av::age_package::observe_package_scalar_states_with_limits(compiled, root, 0, &limits)?
        .into_iter()
        .map(|(name, value)| (name, AceStateValue::from(&value)))
        .collect();
    let timer_enabled = compiled.description.components[root].members.iter()
        .filter(|row| row.role == "timer")
        .enumerate()
        .map(|(index, row)| match row.value {
            Some(_) => ace_av::age_package::evaluate_package_timer(compiled, root, 0, index, &limits),
            None => Ok(true),
        })
        .collect::<Result<Vec<_>, _>>()?;
    assemble(&compiled.description, &compiled.widget_refs, root, revision, view, states, timer_enabled, false)
}

pub(super) fn assemble(
    description: &AgePackageDescription,
    widget_refs: &AgeWidgetRefs,
    root: usize,
    revision: u64,
    view: Vec<WidgetViewItem>,
    states: Vec<(String, AceStateValue)>,
    timer_enabled: Vec<bool>,
    live_children: bool,
) -> Result<PackageFrame, String> {
    let mut frame = ViewBindings {
        description,
        widget_refs,
        nodes: vec![(root, None, Vec::new())],
        live_children,
        mounted: BTreeSet::from([root]),
        active_widgets: BTreeSet::new(),
        actions: Vec::new(),
        child_timers: Vec::new(),
        pointers: BTreeMap::new(),
        image_surfaces: BTreeMap::new(),
        clicks: BTreeMap::new(),
        hits: BTreeMap::new(),
        pens: BTreeMap::new(),
        scenes: BTreeMap::new(),
        links: BTreeMap::new(),
        destinations: Vec::new(),
        widgets: BTreeSet::new(),
        controls: BTreeMap::new(),
        focus_targets: BTreeMap::new(),
        focus_actions: BTreeMap::new(),
        queued_actions: BTreeMap::new(),
        outline: Vec::new(),
    };
    frame.walk(&view, &mut Vec::new())?;
    Ok(PackageFrame {
        revision,
        view,
        states,
        timer_enabled,
        focus_requests: Vec::new(),
        mounted: frame.mounted,
        actions: frame.actions,
        child_timers: frame.child_timers,
        pointers: frame.pointers,
        image_surfaces: frame.image_surfaces,
        clicks: frame.clicks,
        hits: frame.hits,
        pens: frame.pens,
        scenes: frame.scenes,
        links: frame.links,
        destinations: frame.destinations,
        widgets: frame.widgets,
        controls: frame.controls,
        focus_targets: frame.focus_targets,
        focus_actions: frame.focus_actions,
        queued_actions: frame.queued_actions,
    })
}

struct ViewBindings<'a> {
    description: &'a AgePackageDescription,
    widget_refs: &'a AgeWidgetRefs,
    nodes: Vec<(usize, Option<&'a AgeWidgetRef>, Vec<usize>)>,
    live_children: bool,
    mounted: BTreeSet<usize>,
    active_widgets: BTreeSet<usize>,
    actions: Vec<PackageAction>,
    child_timers: Vec<ChildTimer>,
    pointers: BTreeMap<Vec<usize>, PointerEvents>,
    image_surfaces: BTreeMap<Vec<usize>, ImageSurfaceEvents>,
    clicks: BTreeMap<Vec<usize>, usize>,
    /// PLAN-118：canvas onhit 目标（tap 命中 id 经文本事件通道回程序）。
    hits: BTreeMap<Vec<usize>, usize>,
    /// PLAN-118：canvas 笔事件目标（onpenstart/onpenmove/onpenend）。
    pens: BTreeMap<Vec<usize>, (Option<usize>, Option<usize>, Option<usize>)>,
    /// PLAN-118：canvas 场景数据（五平行列表解析后的画布内容，按元素路径）。
    scenes: BTreeMap<Vec<usize>, crate::ui::view::CanvasScene>,
    links: BTreeMap<Vec<usize>, usize>,
    destinations: Vec<String>,
    widgets: BTreeSet<Vec<usize>>,
    controls: BTreeMap<Vec<usize>, PackageControl>,
    focus_targets: BTreeMap<(Vec<usize>, String), Option<String>>,
    focus_actions: BTreeMap<String, [Option<usize>; 2]>,
    queued_actions: BTreeMap<String, Option<usize>>,
    outline: Vec<(usize, Option<String>, usize)>,
}

impl ViewBindings<'_> {
    fn walk(&mut self, items: &[WidgetViewItem], path: &mut Vec<usize>) -> Result<(), String> {
        let mut occurrences = BTreeMap::<usize, usize>::new();
        for (index, item) in items.iter().enumerate() {
            if let WidgetViewItem::Element(element) = item {
                let occurrence = occurrences.entry(element.site).or_default();
                let key = property(element, "key").map(str::to_owned);
                self.outline.push((element.site, key.clone(), if key.is_some() { 0 } else { *occurrence }));
                *occurrence += 1;
                path.push(index);
                self.element(element, path)?;
                path.pop();
                self.outline.pop();
            } else if let WidgetViewItem::Timer(timer) = item {
                if path.is_empty() || self.live_children {
                    continue;
                }
                let (node, message, instance) = self.route(&timer.message)?;
                let component = &self.description.components[node];
                if !component.messages.iter().any(|row| row.name == message && row.payload.is_empty()) {
                    return Err(format!("ace.desktop.widget-timer: '{message}' must take no payload"));
                }
                let timer = ChildTimer {
                    key: ChildTimerKey {
                        owner: ChildTimerOwner::Scenario { path: path.clone(), node, instance: instance.clone() },
                        message,
                        every_ms: timer.every_ms,
                    },
                    node,
                    instance,
                    anchor: component.head_span.start,
                };
                if self.child_timers.iter().any(|existing| existing.key == timer.key) {
                    return Err(format!("ace.desktop.widget-timer: duplicate '{}'", timer.key.message));
                }
                self.child_timers.push(timer);
            }
        }
        Ok(())
    }

    fn element(&mut self, element: &WidgetElement, path: &mut Vec<usize>) -> Result<(), String> {
        let (owner, _, _) = self.nodes.last().expect("view has an owner");
        if let Some(site) = child_site(self.description, self.widget_refs, *owner, element) {
            let (index, child) = self
                .description
                .components
                .iter()
                .enumerate()
                .find(|(_, child)| child.module == site.module && child.name == site.node)
                .ok_or("widget layout references an unknown component")?;
            let requires_unique_state = !child.handler_msgs.is_empty()
                && (!child.fields.is_empty() || child.handler_msgs.len() != child.handler_forwards.len());
            if requires_unique_state && !self.live_children && !self.active_widgets.insert(index) {
                return Err(format!(
                    "ace.desktop.widget-state-sites: repeated stateful component `{}`",
                    child.name,
                ));
            }
            self.mounted.insert(index);
            self.widgets.insert(path.clone());
            self.nodes.push((index, Some(site), path.clone()));
            self.walk(&element.children, path)?;
            self.nodes.pop();
            return Ok(());
        }
        if matches!(element.tag.as_str(), "link" | "sidebar_menu_button") {
            if let Some((_, to)) = element.properties.iter().rev().find(|(key, _)| key == "to") {
                self.links.insert(path.clone(), self.destinations.len());
                self.destinations.push(to.clone());
            }
        }
        if matches!(element.tag.as_str(), "drawer" | "alert-dialog")
            && property(element, "open") != Some("true")
        {
            return Ok(());
        }
        let event_key = if element.tag == "drawer" { "onclose" } else { "onclick" };
        let disabled = property(element, "disabled") == Some("true");
        if let Some(event) = element.events.iter().find(|event| event.key.eq_ignore_ascii_case(event_key)) {
            let target = self.action(event, disabled)?;
            if !disabled && event_key == "onclick" {
                self.bind_queued_action(target, event_key, None);
            }
            self.clicks.insert(path.clone(), target);
        }
        if matches!(element.tag.as_str(), "image_surface" | "image-surface" | "imagesurface") {
            self.image_surface_bindings(element, path, disabled)?;
        }
        // PLAN-118：canvas onhit —— tap 命中元素 id 经文本事件通道回事件程序
        // （`.PaintHit(id)` 范式；命中判定在渲染层 canvas_hit_test）。
        if element.tag == "canvas" {
            if let Some(event) = element.events.iter().find(|e| e.key.eq_ignore_ascii_case("onhit")) {
                let target = self.action(event, disabled)?;
                self.hits.insert(path.clone(), target);
            }
            let mut pen = (None, None, None);
            for event in &element.events {
                let key = event.key.to_ascii_lowercase();
                let slot = match key.as_str() {
                    "onpenstart" => &mut pen.0,
                    "onpenmove" => &mut pen.1,
                    "onpenend" => &mut pen.2,
                    _ => continue,
                };
                if slot.is_none() {
                    *slot = Some(self.action(event, disabled)?);
                }
            }
            if pen != (None, None, None) {
                self.pens.insert(path.clone(), pen);
            }
        }
        if !disabled {
            self.control(element, path)?;
        }
        if !disabled && element.tag == "mouse-area" {
            let mut pointer = PointerEvents::default();
            for event in &element.events {
                let key = event.key.to_ascii_lowercase();
                let slot = match key.as_str() {
                    "onmouseenter" => &mut pointer.enter,
                    "onmouseleave" => &mut pointer.leave,
                    "onmousemove" => &mut pointer.motion,
                    _ => continue,
                };
                if slot.is_some() {
                    return Err(format!("ace.desktop.pointer: repeated binding for `{key}`"));
                }
                let target = self.action(event, false)?;
                if key == "onmousemove" {
                    let action = &self.actions[target];
                    let component = &self.description.components[action.node];
                    let typed = component.messages.iter().any(|message| {
                        message.name == action.message && message.payload.len() == 2
                            && message.payload.iter().all(|ty| matches!(ty.as_str(), "float" | "f32"))
                    });
                    if !event.args.is_empty() || !typed {
                        return Err(format!("ace.desktop.pointer: `{}` needs two f32 coordinates", action.message));
                    }
                }
                *slot = Some(target);
            }
            self.pointers.insert(path.clone(), pointer);
        }
        self.walk(&element.children, path)
    }

    fn image_surface_bindings(
        &mut self,
        element: &WidgetElement,
        path: &[usize],
        disabled: bool,
    ) -> Result<(), String> {
        let mut bindings = ImageSurfaceEvents::default();
        let source = property(element, "src").unwrap_or_default();
        let identity = serde_json::to_string(&(source, property(element, "cache_identity"))).unwrap();
        for event in &element.events {
            let key = event.key.to_ascii_lowercase();
            let slot = match key.as_str() {
                "onload" | "onloaded" | "on_loaded" => &mut bindings.loaded,
                "onerror" | "on_error" => &mut bindings.error,
                "onwheel" if !disabled => &mut bindings.wheel,
                "onpan" if !disabled => &mut bindings.pan,
                "ondblclick" if !disabled => &mut bindings.double_click,
                _ => continue,
            };
            if slot.is_some() {
                return Err(format!("ace.desktop.image-surface: repeated binding for `{key}`"));
            }
            let target = self.action(event, false)?;
            self.bind_queued_action(target, &key, Some(&identity));
            *slot = Some(target);
        }
        self.image_surfaces.insert(path.to_vec(), bindings);
        Ok(())
    }

    fn action(&mut self, event: &WidgetEvent, disabled: bool) -> Result<usize, String> {
        let (node, message, instance) = self.route(&event.message)?;
        let target = self.actions.len();
        self.actions.push(PackageAction {
            source: event.clone(),
            node,
            instance,
            message,
            anchor: self.description.components[node].head_span,
            input_value: false,
            disabled,
            queued_control: None,
        });
        Ok(target)
    }

    fn control(&mut self, element: &WidgetElement, path: &[usize]) -> Result<(), String> {
        let mut control = PackageControl::default();
        for event in &element.events {
            let name = event.key.to_ascii_lowercase();
            let mut parts = name.split('.');
            match parts.next() {
                Some("oninput") if element.tag == "input" && name == "oninput" => {
                    control.input = Some(self.editor_action(event, true)?);
                }
                Some("onenter") if element.tag == "input" && name == "onenter" => {
                    control.submit = Some(self.editor_action(event, false)?);
                }
                Some("oncontextmenu") => control.context_menu = Some(self.action(event, false)?),
                Some("ondblclick") if !self.image_surfaces.contains_key(path) => {
                    let target = self.action(event, false)?;
                    if self.outline.iter().any(|(_, key, _)| key.is_some()) {
                        self.bind_queued_action(target, &event.key, None);
                    }
                    control.double_click = Some(target);
                }
                Some("onkeydown") => {
                    if let Some(key) = parts.find(|key| matches!(*key,
                        "left" | "right" | "up" | "down" | "home" | "end" | "enter" | "space" | "escape"))
                    {
                        control.keys.push((key.to_owned(), self.action(event, false)?));
                    }
                }
                _ => {}
            }
        }
        let focusable = matches!(element.tag.as_str(), "button" | "alert-dialog-action" | "alert-dialog-cancel")
            && self.clicks.contains_key(path)
            || element.tag == "input"
            || property(element, "tabindex").is_some_and(|value| value.parse::<i32>().is_ok())
            || !control.keys.is_empty();
        if focusable {
            let (_, _, scope) = self.nodes.last().expect("view has an owner");
            let id = property(element, "id").filter(|id| !id.is_empty());
            let on_focus = element.events.iter().find(|event| event.key.eq_ignore_ascii_case("onfocus"));
            let on_blur = element.events.iter().find(|event| event.key.eq_ignore_ascii_case("onblur"));
            if id.is_some() || !control.keys.is_empty() || on_focus.is_some() || on_blur.is_some() {
                let focus = serde_json::to_string(&(&self.outline, id)).unwrap();
                if let Some(id) = id {
                    self.focus_targets.entry((scope.clone(), id.to_owned()))
                        .and_modify(|previous| *previous = None)
                        .or_insert_with(|| Some(focus.clone()));
                }
                let blur = on_blur.map(|event| self.action(event, false)).transpose()?;
                let gain = on_focus.map(|event| self.action(event, false)).transpose()?;
                control.on_blur = blur.is_some();
                control.on_focus = gain.is_some();
                if control.on_blur || control.on_focus {
                    self.focus_actions.insert(focus.clone(), [blur, gain]);
                }
                control.focus = Some(focus);
            }
        }
        if control.focus.is_some() || control.context_menu.is_some() || control.double_click.is_some()
            || control.input.is_some() || control.submit.is_some()
        {
            self.controls.insert(path.to_vec(), control);
        }
        Ok(())
    }

    fn editor_action(&mut self, event: &WidgetEvent, input: bool) -> Result<usize, String> {
        let target = self.action(event, false)?;
        let action = &mut self.actions[target];
        if input && event.args.is_empty() {
            let component = &self.description.components[action.node];
            if let Some(message) = component.messages.iter().find(|m| m.name == action.message) {
                if !message.payload.is_empty() {
                    let [kind] = message.payload.as_slice() else {
                        return Err(format!("ace.desktop.widget-input: '{}' requires one text argument", action.message));
                    };
                    if !kind.eq_ignore_ascii_case("str") {
                        return Err(format!("ace.desktop.widget-input: '{}' requires a text argument", action.message));
                    }
                    action.input_value = true;
                }
            }
        }
        self.bind_queued_action(target, &event.key, None);
        Ok(target)
    }

    fn bind_queued_action(&mut self, target: usize, event: &str, source: Option<&str>) {
        let key = serde_json::to_string(&(&self.outline, event, source)).unwrap();
        self.actions[target].queued_control = Some(key.clone());
        self.queued_actions.entry(key).and_modify(|previous| *previous = None).or_insert(Some(target));
    }

    fn route(&self, message: &str) -> Result<(usize, String, Vec<usize>), String> {
        let mut message = message;
        for (node, site, position) in self.nodes.iter().rev() {
            let component = &self.description.components[*node];
            let forward = component.handler_forwards.get(message);
            if forward.is_none() && component
                .handler_msgs
                .iter()
                .any(|handler| handler == message)
            {
                return Ok((*node, message.to_owned(), position.clone()));
            }
            message = site
                .and_then(|site| {
                    if let Some(event) = site.events.iter().find(|event| event.name == message) {
                        return Some(event.target.as_str());
                    }
                    site.properties.iter().find_map(|prop| {
                        let AgeWidgetProperty::Message { name, target, .. } = prop else {
                            return None;
                        };
                        let matches = match forward {
                            Some(channel) => name == channel,
                            None => channel_pascal(name) == message,
                        };
                        matches.then_some(target.as_str())
                    })
                })
                .ok_or_else(|| {
                    format!(
                "ace.desktop.widget-event-route: `{}` has no handler or callback for `{message}`",
                component.name,
            )
                })?;
        }
        Err(format!(
            "ace.desktop.widget-event-route: unresolved callback `{message}`"
        ))
    }
}

fn property<'a>(element: &'a WidgetElement, name: &str) -> Option<&'a str> {
    element.properties.iter().rev().find_map(|(key, value)| (key == name).then_some(value.as_str()))
}

pub(super) fn child_site<'a>(
    description: &'a AgePackageDescription,
    widget_refs: &'a AgeWidgetRefs,
    owner: usize,
    element: &WidgetElement,
) -> Option<&'a AgeWidgetRef> {
    let component = &description.components[owner];
    widget_refs
        .get(&(component.module.clone(), component.name.clone()))?
        .iter()
        .find(|site| site.span.start == element.site && site.authored_node == element.tag)
}
