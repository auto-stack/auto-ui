use std::collections::{BTreeMap, BTreeSet};

use ace_av::age_package::{WidgetElement, WidgetEvent, WidgetViewItem};
use ace_core::source::age_package::{AgePackageCompilation, AgeWidgetProperty, AgeWidgetRef};

use crate::ui::ace_vue_script::channel_pascal;

use super::AceStateValue;

/// A rendered package and the events offered by that exact revision.
#[derive(Debug)]
pub struct PackageFrame {
    pub revision: u64,
    pub view: Vec<WidgetViewItem>,
    pub(super) states: Vec<(String, AceStateValue)>,
    pub(super) mounted: BTreeSet<usize>,
    pub(super) actions: Vec<PackageAction>,
    pub(super) timers: Vec<PackageTimer>,
    clicks: BTreeMap<Vec<usize>, usize>,
    pointers: BTreeMap<Vec<usize>, PointerEvents>,
    links: BTreeMap<Vec<usize>, usize>,
    destinations: Vec<String>,
    widgets: BTreeSet<Vec<usize>>,
}

#[derive(Debug)]
pub struct PackageAction {
    pub source: WidgetEvent,
    pub(super) node: usize,
    pub(super) message: String,
    pub(super) anchor: ace_core::atom_text::Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PackageTimer {
    pub owner_path: Vec<usize>,
    pub node: usize,
    pub message: String,
    pub every_ms: u32,
    pub anchor: usize,
}

#[derive(Debug, Default)]
pub(crate) struct PointerEvents {
    pub(crate) enter: Option<usize>,
    pub(crate) leave: Option<usize>,
    pub(crate) motion: Option<usize>,
}

impl PackageFrame {
    pub fn actions(&self) -> &[PackageAction] {
        &self.actions
    }

    pub(crate) fn click(&self, path: &[usize]) -> Option<usize> {
        self.clicks.get(path).copied()
    }

    pub(crate) fn pointer(&self, path: &[usize]) -> Option<&PointerEvents> {
        self.pointers.get(path)
    }

    pub(crate) fn link(&self, path: &[usize]) -> Option<usize> {
        self.links.get(path).map(|index| self.actions.len() + index)
    }

    pub(crate) fn navigation_target(&self, target: usize) -> Option<&str> {
        let index = target.checked_sub(self.actions.len())?;
        self.destinations.get(index).map(String::as_str)
    }

    pub(crate) fn accepts_target(&self, target: usize) -> bool {
        target < self.actions.len() + self.destinations.len()
    }

    pub(crate) fn action_message(&self, target: usize) -> Option<&str> {
        self.actions.get(target).map(|action| action.message.as_str())
    }

    pub(crate) fn model_values(&self) -> &[(String, AceStateValue)] {
        &self.states
    }

    pub(crate) fn is_widget(&self, path: &[usize]) -> bool {
        self.widgets.contains(path)
    }
}

pub(super) fn render(
    compiled: &AgePackageCompilation,
    root: usize,
    revision: u64,
) -> Result<PackageFrame, String> {
    let limits = super::view_limits();
    let view = ace_av::age_package::evaluate_package_view(compiled, 0, &limits)?;
    let mut frame = ViewBindings {
        compiled,
        nodes: vec![(root, None)],
        mounted: BTreeSet::from([root]),
        active_widgets: BTreeSet::new(),
        actions: Vec::new(),
        timers: Vec::new(),
        clicks: BTreeMap::new(),
        pointers: BTreeMap::new(),
        links: BTreeMap::new(),
        destinations: Vec::new(),
        widgets: BTreeSet::new(),
    };
    frame.walk(&view, &mut Vec::new())?;
    Ok(PackageFrame {
        revision,
        view,
        states: Vec::new(),
        mounted: frame.mounted,
        actions: frame.actions,
        timers: frame.timers,
        clicks: frame.clicks,
        pointers: frame.pointers,
        links: frame.links,
        destinations: frame.destinations,
        widgets: frame.widgets,
    })
}

struct ViewBindings<'a> {
    compiled: &'a AgePackageCompilation,
    nodes: Vec<(usize, Option<&'a AgeWidgetRef>)>,
    mounted: BTreeSet<usize>,
    active_widgets: BTreeSet<usize>,
    actions: Vec<PackageAction>,
    timers: Vec<PackageTimer>,
    clicks: BTreeMap<Vec<usize>, usize>,
    pointers: BTreeMap<Vec<usize>, PointerEvents>,
    links: BTreeMap<Vec<usize>, usize>,
    destinations: Vec<String>,
    widgets: BTreeSet<Vec<usize>>,
}

impl ViewBindings<'_> {
    fn walk(&mut self, items: &[WidgetViewItem], path: &mut Vec<usize>) -> Result<(), String> {
        for (index, item) in items.iter().enumerate() {
            match item {
                WidgetViewItem::Element(element) => {
                    path.push(index);
                    self.element(element, path)?;
                    path.pop();
                }
                WidgetViewItem::Timer(timer) => {
                    let (node, message) = self.route(&timer.message)?;
                    let component = &self.compiled.description.components[node];
                    if !component.messages.iter().any(|row| row.name == message && row.payload.is_empty()) {
                        return Err(format!("ace.desktop.widget-timer: '{message}' must take no payload"));
                    }
                    let timer = PackageTimer {
                        owner_path: path.clone(),
                        node,
                        message,
                        every_ms: timer.every_ms,
                        anchor: component.head_span.start,
                    };
                    if self.timers.contains(&timer) {
                        return Err(format!("ace.desktop.widget-timer: duplicate '{}'", timer.message));
                    }
                    self.timers.push(timer);
                }
                WidgetViewItem::Text(_) => {}
            }
        }
        Ok(())
    }

    fn element(&mut self, element: &WidgetElement, path: &mut Vec<usize>) -> Result<(), String> {
        let (owner, _) = self.nodes.last().expect("view has an owner");
        if let Some(site) = child_site(self.compiled, *owner, element) {
            let (index, child) = self
                .compiled
                .description
                .components
                .iter()
                .enumerate()
                .find(|(_, child)| child.module == site.module && child.name == site.node)
                .ok_or("widget layout references an unknown component")?;
            let requires_unique_state = !child.handler_msgs.is_empty()
                && (!child.fields.is_empty() || child.handler_msgs.len() != child.handler_forwards.len());
            if requires_unique_state && !self.active_widgets.insert(index) {
                return Err(format!(
                    "ace.desktop.widget-state-sites: repeated stateful component `{}`",
                    child.name,
                ));
            }
            self.mounted.insert(index);
            self.widgets.insert(path.clone());
            self.nodes.push((index, Some(site)));
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
        let event_key = if element.tag == "drawer" { "onclose" } else { "onclick" };
        if let Some(event) = element.events.iter().find(|event| event.key.eq_ignore_ascii_case(event_key)) {
            let target = self.bind_action(event)?;
            self.clicks.insert(path.clone(), target);
        }
        if element.tag == "mouse-area" {
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
                let target = self.bind_action(event)?;
                if key == "onmousemove" {
                    let action = &self.actions[target];
                    let component = &self.compiled.description.components[action.node];
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

    fn bind_action(&mut self, event: &WidgetEvent) -> Result<usize, String> {
        let (node, message) = self.route(&event.message)?;
        let target = self.actions.len();
        self.actions.push(PackageAction {
            source: event.clone(),
            node,
            message,
            anchor: self.compiled.description.components[node].head_span,
        });
        Ok(target)
    }

    fn route(&self, message: &str) -> Result<(usize, String), String> {
        let mut message = message;
        for &(node, site) in self.nodes.iter().rev() {
            let component = &self.compiled.description.components[node];
            let forward = component.handler_forwards.get(message);
            if forward.is_none() && component
                .handler_msgs
                .iter()
                .any(|handler| handler == message)
            {
                return Ok((node, message.to_owned()));
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

pub(super) fn child_site<'a>(
    compiled: &'a AgePackageCompilation,
    owner: usize,
    element: &WidgetElement,
) -> Option<&'a AgeWidgetRef> {
    let component = &compiled.description.components[owner];
    compiled
        .widget_refs
        .get(&(component.module.clone(), component.name.clone()))?
        .iter()
        .find(|site| site.span.start == element.site && site.authored_node == element.tag)
}
