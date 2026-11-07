//! Windows system accessibility for the production VM window.
//! No VM object or debug/MCP tree crosses the platform adapter boundary.
use super::{AbstractView, IcedMessage};
use crate::ui::dynamic::DynamicComponent;
use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Rect, Role,
    Toggled, TreeId, TreeInfo, TreeUpdate,
};
use accesskit_windows::SubclassingAdapter;
use futures::{channel::mpsc, StreamExt};
use iced::advanced::{
    widget::{tree, Operation, Tree},
    Layout, Widget,
};
use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use raw_window_handle::RawWindowHandle;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
    sync::{Arc, Mutex, OnceLock},
};

const ROOT: NodeId = NodeId(1);
const QUEUE_CAP: usize = 128;
const TICK: &str = "__native_accessibility_tick";
const ACTION: &str = "__native_accessibility_action";
type Shared = Arc<Mutex<WindowState>>;
static WINDOWS: OnceLock<Mutex<HashMap<iced::window::Id, Shared>>> = OnceLock::new();
fn windows() -> &'static Mutex<HashMap<iced::window::Id, Shared>> {
    WINDOWS.get_or_init(Default::default)
}
fn shared(window: iced::window::Id) -> Shared {
    windows()
        .lock()
        .unwrap()
        .entry(window)
        .or_insert_with(|| Arc::new(Mutex::new(WindowState::new())))
        .clone()
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Owner {
    root: u64,
    identity: Option<String>,
    receiver: u64,
}
impl Owner {
    fn alive(&self, component: &DynamicComponent) -> bool {
        self.root == component.bridge().state_obj_id()
            && self
                .identity
                .as_ref()
                .map_or(self.receiver == self.root, |identity| {
                    component.bridge().child_instance_state_id(identity) == Some(self.receiver)
                })
    }
}
#[derive(Clone)]
struct Entry {
    node: Node,
    owner: Owner,
    click: Option<IcedMessage>,
    change: Option<IcedMessage>,
    focus: Option<iced::widget::Id>,
    bounds: Option<Rectangle>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Key {
    path: Vec<usize>,
    parent: u64,
    receiver: u64,
    role: Role,
    event: String,
    occurrence: usize,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum Requested {
    Click,
    Focus,
    Value(String),
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Envelope {
    node: u64,
    action: Requested,
}
struct WindowState {
    name: String,
    nodes: BTreeMap<u64, Entry>,
    keys: HashMap<Key, u64>,
    next: u64,
    focus: NodeId,
    roots: Vec<NodeId>,
    ready: bool,
    scale: f64,
    wake_pending: bool,
    sender: mpsc::UnboundedSender<()>,
    receiver: Option<mpsc::UnboundedReceiver<()>>,
    actions: VecDeque<Envelope>,
}
impl WindowState {
    fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        Self {
            name: String::new(),
            nodes: BTreeMap::new(),
            keys: HashMap::new(),
            next: 2,
            focus: ROOT,
            roots: vec![],
            ready: false,
            scale: 1.0,
            wake_pending: false,
            sender,
            receiver: Some(receiver),
            actions: VecDeque::new(),
        }
    }
    fn wake(&mut self) {
        if !self.wake_pending {
            self.wake_pending = true;
            let _ = self.sender.unbounded_send(());
        }
    }
    fn snapshot(&self) -> Option<TreeUpdate> {
        if !self.ready {
            return None;
        }
        let mut root = Node::new(Role::Window);
        root.set_label(self.name.clone());
        root.set_children(self.roots.clone());
        let mut nodes = vec![(ROOT, root)];
        for (id, entry) in &self.nodes {
            let mut node = entry.node.clone();
            if let Some(bounds) = entry.bounds {
                node.set_bounds(Rect::new(
                    bounds.x as f64 * self.scale,
                    bounds.y as f64 * self.scale,
                    (bounds.x + bounds.width) as f64 * self.scale,
                    (bounds.y + bounds.height) as f64 * self.scale,
                ));
            } else {
                node.set_hidden();
            }
            nodes.push((NodeId(*id), node));
        }
        Some(TreeUpdate {
            nodes,
            tree: Some(TreeInfo::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus: if self.nodes.contains_key(&self.focus.0) {
                self.focus
            } else {
                ROOT
            },
        })
    }
}
struct Activate(Shared);
impl ActivationHandler for Activate {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.0.lock().unwrap().snapshot()
    }
}
struct Actions(Shared);
impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        if request.target_tree != TreeId::ROOT {
            return;
        }
        let action = match (request.action, request.data) {
            (Action::Click, _) => Requested::Click,
            (Action::Focus, _) => Requested::Focus,
            (Action::SetValue, Some(ActionData::Value(value))) => Requested::Value(value.into()),
            _ => return,
        };
        let mut state = self.0.lock().unwrap();
        if state.actions.len() >= QUEUE_CAP {
            log::warn!("native accessibility action queue full");
            return;
        }
        state.actions.push_back(Envelope {
            node: request.target_node.0,
            action,
        });
        state.wake();
    }
}
thread_local! {
    static ADAPTERS: RefCell<HashMap<iced::window::Id, SubclassingAdapter>> = RefCell::new(HashMap::new());
    static BUILD: RefCell<Option<Build>> = const { RefCell::new(None) };
}

pub(super) fn prepare(window: iced::window::Id) {
    shared(window);
}

/// Runs on the owning window thread, before first show/focus.
fn install(window: iced::window::Id, native: &dyn iced_runtime::window::Window) {
    let Ok(handle) = native.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let state = shared(window);
    ADAPTERS.with(|adapters| {
        adapters.borrow_mut().entry(window).or_insert_with(|| {
            SubclassingAdapter::new(
                accesskit_windows::HWND(handle.hwnd.get() as _),
                Activate(state.clone()),
                Actions(state),
            )
        });
    });
}
/// Preserve the caller's hidden/pixels intent after installing the provider.
pub(super) fn install_task(window: iced::window::Id, show: bool) -> iced::Task<iced::window::Id> {
    iced::window::run(window, move |native| install(window, native)).then(move |_| {
        if show {
            iced::window::set_mode(window, iced::window::Mode::Windowed)
                .chain(iced::Task::done(window))
        } else {
            iced::Task::done(window)
        }
    })
}
fn publish(window: iced::window::Id) {
    let alive = windows()
        .lock()
        .unwrap()
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let state = windows().lock().unwrap().get(&window).cloned();
    let tree = state.and_then(|state| state.lock().unwrap().snapshot());
    // Snapshot locks and the adapter RefCell are released before raising events.
    let events = ADAPTERS.with(|adapters| {
        let mut adapters = adapters.borrow_mut();
        adapters.retain(|id, _| alive.contains(id));
        tree.and_then(|tree| {
            adapters
                .get_mut(&window)
                .and_then(|adapter| adapter.update_if_active(|| tree))
        })
    });
    if let Some(events) = events {
        events.raise();
    }
}
pub(super) fn remove(window: iced::window::Id) {
    let others = {
        let mut registry = windows().lock().unwrap();
        registry.remove(&window);
        registry.values().cloned().collect::<Vec<_>>()
    };
    // Remaining windows wake their owning thread to release closed adapters.
    // If the last window closed, thread-local adapters die with its event loop.
    for other in others {
        other.lock().unwrap().wake();
    }
}
pub(super) fn subscription(
    window: iced::window::Id,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced::Subscription::run_with(window, |window| {
        let window = *window;
        let receiver = windows()
            .lock()
            .unwrap()
            .get(&window)
            .cloned()
            .and_then(|state| state.lock().unwrap().receiver.take());
        let stream: futures::stream::BoxStream<'static, _> = match receiver {
            Some(receiver) => receiver
                .map(move |_| {
                    crate::ui::session::DesktopMessage::Window(
                        window,
                        IcedMessage {
                            widget: String::new(),
                            event: TICK.into(),
                            input_value: None,
                        },
                    )
                })
                .boxed(),
            None => futures::stream::pending().boxed(),
        };
        stream
    })
}
pub(super) fn is_tick(message: &IcedMessage) -> bool {
    message.event == TICK
}
pub(super) fn tick(window: iced::window::Id) -> iced::Task<IcedMessage> {
    let Some(state) = windows().lock().unwrap().get(&window).cloned() else {
        return iced::Task::none();
    };
    let actions = {
        let mut state = state.lock().unwrap();
        state.wake_pending = false;
        state.actions.drain(..).collect::<Vec<_>>()
    };
    let mut tasks = actions
        .into_iter()
        .map(|action| {
            iced::Task::done(IcedMessage {
                widget: String::new(),
                event: ACTION.into(),
                input_value: serde_json::to_string(&action).ok(),
            })
        })
        .collect::<Vec<_>>();
    tasks.push(iced::window::scale_factor(window).then(move |scale| {
        state.lock().unwrap().scale = scale as f64;
        iced::window::run(window, move |_| publish(window)).discard()
    }));
    iced::Task::batch(tasks)
}
pub(super) enum Dispatch {
    Message(IcedMessage),
    Focus(iced::widget::Id),
    Rejected,
}
pub(super) fn resolve(
    window: iced::window::Id,
    component: &DynamicComponent,
    message: &IcedMessage,
) -> Option<Dispatch> {
    if message.event != ACTION {
        return None;
    }
    let result = (|| {
        let envelope: Envelope = serde_json::from_str(message.input_value.as_deref()?).ok()?;
        let state = windows().lock().unwrap().get(&window).cloned()?;
        let state = state.lock().unwrap();
        let entry = state.nodes.get(&envelope.node)?;
        if !entry.owner.alive(component) || entry.node.is_disabled() || entry.bounds.is_none() {
            return None;
        }
        match envelope.action {
            Requested::Click => entry.click.clone().map(Dispatch::Message),
            Requested::Focus => entry.focus.clone().map(Dispatch::Focus),
            Requested::Value(value) => entry.change.clone().map(|mut message| {
                message.input_value = Some(value);
                Dispatch::Message(message)
            }),
        }
    })();
    Some(result.unwrap_or(Dispatch::Rejected))
}

struct Build {
    state: Shared,
    root: u64,
    root_widget: String,
    owners: HashMap<String, u64>,
    /// PLAN-069 F3: identity → widget TYPE (registered name). Identity strings
    /// embed outlet routes, so suffix parsing cannot recover the type.
    owner_types: HashMap<String, String>,
    stack: Vec<u64>,
    nodes: BTreeMap<u64, Entry>,
    keys: HashMap<Key, u64>,
    roots: Vec<NodeId>,
    occurrences: HashMap<(u64, Role, String), usize>,
}
pub(super) fn with_scope<R>(
    window: iced::window::Id,
    component: &DynamicComponent,
    f: impl FnOnce() -> R,
) -> R {
    let Some(state) = windows().lock().unwrap().get(&window).cloned() else {
        return f();
    };
    let name = super::host::window_title(format!("Auto - {}", component.widget_name()));
    {
        let mut state = state.lock().unwrap();
        if state.name != name {
            state.name = name;
            state.wake();
        }
    }
    let owners = component
        .bridge()
        .child_instance_identities()
        .into_iter()
        .filter_map(|id| {
            component
                .bridge()
                .child_instance_state_id(&id)
                .map(|state| (id, state))
        })
        .collect();
    let owner_types = component
        .bridge()
        .child_instance_identities()
        .into_iter()
        .filter_map(|id| {
            component
                .bridge()
                .child_instance_widget_name(&id)
                .map(|ty| (id, ty))
        })
        .collect();
    let build = Build {
        state,
        root: component.bridge().state_obj_id(),
        root_widget: component.widget_name().to_string(),
        owners,
        owner_types,
        stack: vec![],
        nodes: BTreeMap::new(),
        keys: HashMap::new(),
        roots: vec![],
        occurrences: HashMap::new(),
    };
    let previous = BUILD.with(|slot| slot.replace(Some(build)));
    struct Restore(Option<Build>);
    impl Drop for Restore {
        fn drop(&mut self) {
            BUILD.with(|slot| {
                slot.replace(self.0.take());
            });
        }
    }
    let _restore = Restore(previous);
    f()
}
fn bind(build: &Build, message: Option<&IcedMessage>) -> (Owner, Option<IcedMessage>) {
    let root = Owner {
        root: build.root,
        receiver: build.root,
        identity: None,
    };
    let Some(message) = message else {
        return (root, None);
    };
    let (widget, identity) = message
        .widget
        .split_once('\u{1}')
        .map_or((message.widget.as_str(), None), |(widget, id)| {
            (widget, Some(id))
        });
    let resolved = identity
        .and_then(|id| {
            build
                .owners
                .get(id)
                .map(|receiver| (id.to_string(), *receiver))
        })
        .or_else(|| {
            // An explicit identity is authoritative, including when it has
            // already been released. Never retarget it to a same-type sibling.
            if identity.is_some() {
                return None;
            }
            let mut matches = build
                .owners
                .iter()
                .filter(|(id, _)| build.owner_types.get(*id).map(String::as_str) == Some(widget));
            let only = matches.next()?;
            if matches.next().is_some() {
                None
            } else {
                Some((only.0.clone(), *only.1))
            }
        });
    if let Some((identity, receiver)) = resolved {
        let mut message = message.clone();
        message.widget = format!("{widget}\u{1}{identity}");
        (
            Owner {
                root: build.root,
                identity: Some(identity),
                receiver,
            },
            Some(message),
        )
    } else if identity.is_none() && (widget.is_empty() || widget == build.root_widget) {
        (root, Some(message.clone()))
    } else {
        (root, None)
    }
}
fn label(view: &AbstractView<IcedMessage>) -> String {
    match view {
        AbstractView::Text { content, .. } => content.clone(),
        AbstractView::Rich { spans, .. } => spans.iter().map(|s| s.content.as_str()).collect(),
        AbstractView::Row { children, .. } | AbstractView::Column { children, .. } => children
            .iter()
            .map(label)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            label(child)
        }
        _ => String::new(),
    }
}
pub(super) struct Ticket {
    state: Shared,
    id: u64,
    focus: Option<iced::widget::Id>,
    click: Option<IcedMessage>,
    button_focus: bool,
}
pub(super) fn enter(view: &AbstractView<IcedMessage>, path: &[usize]) -> Option<Ticket> {
    BUILD.with(|slot| {
        let mut slot = slot.borrow_mut();
        let build = slot.as_mut()?;
        let (role, name, value, click, change, focus, disabled, toggled, selected, button_focus) =
            match view {
                AbstractView::Button {
                    label: name,
                    onclick,
                    disabled,
                    content,
                    ..
                } => (
                    Role::Button,
                    if name.is_empty() {
                        content.as_ref().map(|v| label(v)).unwrap_or_default()
                    } else {
                        name.clone()
                    },
                    None,
                    Some(onclick),
                    None,
                    None,
                    *disabled,
                    None,
                    None,
                    true,
                ),
                AbstractView::Input {
                    placeholder,
                    value,
                    on_change,
                    on_submit,
                    width,
                    password,
                    disabled,
                    ..
                } => {
                    let primary = on_change.as_ref().or(on_submit.as_ref());
                    let focus = super::style_widgets::derive_input_id(
                        primary.map(|m| (m.widget.as_str(), m.event.as_str())),
                        placeholder,
                        *width,
                        *password,
                    );
                    (
                        Role::TextInput,
                        placeholder.clone(),
                        if *password { None } else { Some(value.clone()) },
                        None,
                        on_change.as_ref(),
                        Some(focus),
                        *disabled,
                        None,
                        None,
                        false,
                    )
                }
                AbstractView::Textarea {
                    placeholder,
                    value,
                    on_change,
                    on_submit,
                    disabled,
                    ..
                } => {
                    let key = on_change
                        .as_ref()
                        .or(on_submit.as_ref())
                        .map(|m| format!("{}_{}", m.widget, m.event))
                        .unwrap_or_else(|| format!("__textarea_{}", placeholder.len()));
                    (
                        Role::MultilineTextInput,
                        placeholder.clone(),
                        Some(value.clone()),
                        None,
                        on_change.as_ref(),
                        Some(iced::widget::Id::from(format!("textarea_{key}"))),
                        *disabled,
                        None,
                        None,
                        false,
                    )
                }
                AbstractView::Text { content, .. } => (
                    Role::Label,
                    String::new(),
                    Some(content.clone()),
                    None,
                    None,
                    None,
                    false,
                    None,
                    None,
                    false,
                ),
                AbstractView::Rich { .. } => (
                    Role::Label,
                    String::new(),
                    Some(label(view)),
                    None,
                    None,
                    None,
                    false,
                    None,
                    None,
                    false,
                ),
                AbstractView::Checkbox {
                    label,
                    is_checked,
                    on_toggle,
                    ..
                } => (
                    Role::CheckBox,
                    label.clone(),
                    None,
                    on_toggle.as_ref(),
                    None,
                    None,
                    false,
                    Some(*is_checked),
                    None,
                    true,
                ),
                AbstractView::Radio {
                    label,
                    is_selected,
                    on_select,
                    ..
                } => (
                    Role::RadioButton,
                    label.clone(),
                    None,
                    on_select.as_ref(),
                    None,
                    None,
                    false,
                    None,
                    Some(*is_selected),
                    true,
                ),
                AbstractView::Row { onclick, .. } | AbstractView::Column { onclick, .. }
                    if onclick.is_some() =>
                {
                    (
                        Role::Button,
                        label(view),
                        None,
                        onclick.as_ref(),
                        None,
                        None,
                        false,
                        None,
                        None,
                        true,
                    )
                }
                AbstractView::List { .. } => (
                    Role::List,
                    String::new(),
                    None,
                    None,
                    None,
                    None,
                    false,
                    None,
                    None,
                    false,
                ),
                _ => (
                    Role::GenericContainer,
                    String::new(),
                    None,
                    None,
                    None,
                    None,
                    false,
                    None,
                    None,
                    false,
                ),
            };
        let primary = click.or(change);
        let (owner, bound) = bind(build, primary);
        let owner_bound = primary.is_none() || bound.is_some();
        let (_, click) = bind(build, click);
        let (_, change) = bind(build, change);
        let event = primary
            .map(|m| format!("{}:{}", m.widget, m.event))
            .unwrap_or_default();
        let occurrence = if primary.is_some() {
            let next = build
                .occurrences
                .entry((owner.receiver, role, event.clone()))
                .or_default();
            let current = *next;
            *next += 1;
            current
        } else {
            0
        };
        let key = Key {
            path: if primary.is_some() {
                vec![]
            } else {
                path.to_vec()
            },
            parent: if primary.is_some() {
                0
            } else {
                build.stack.last().copied().unwrap_or(1)
            },
            receiver: owner.receiver,
            role,
            event,
            occurrence,
        };
        let id = {
            let mut state = build.state.lock().unwrap();
            if let Some(id) = state.keys.get(&key) {
                *id
            } else {
                let id = state.next;
                state.next += 1;
                id
            }
        };
        build.keys.insert(key, id);
        let focus = if disabled || !owner_bound {
            None
        } else {
            focus.or_else(|| {
                button_focus.then(|| iced::widget::Id::from(format!("native_accessibility_{id}")))
            })
        };
        let mut node = Node::new(role);
        if !name.is_empty() {
            node.set_label(name);
        }
        if let Some(value) = value {
            node.set_value(value);
        }
        if disabled {
            node.set_disabled();
        } else {
            if click.is_some() {
                node.add_action(Action::Click);
            }
            if change.is_some() {
                node.add_action(Action::SetValue);
            }
            if focus.is_some() {
                node.add_action(Action::Focus);
            }
        }
        if let Some(checked) = toggled {
            node.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if let Some(selected) = selected {
            node.set_selected(selected);
        }
        let bounds = build
            .state
            .lock()
            .unwrap()
            .nodes
            .get(&id)
            .and_then(|entry| entry.bounds);
        build.nodes.insert(
            id,
            Entry {
                node,
                owner,
                click: click.clone(),
                change,
                focus: focus.clone(),
                bounds,
            },
        );
        if let Some(parent) = build.stack.last() {
            build
                .nodes
                .get_mut(parent)
                .unwrap()
                .node
                .push_child(NodeId(id));
        } else {
            build.roots.push(NodeId(id));
        }
        build.stack.push(id);
        Some(Ticket {
            state: build.state.clone(),
            id,
            focus,
            click: if disabled { None } else { click },
            button_focus: button_focus && !disabled && owner_bound,
        })
    })
}
pub(super) fn wrap(
    ticket: Option<Ticket>,
    child: Element<'static, IcedMessage>,
) -> Element<'static, IcedMessage> {
    let Some(ticket) = ticket else { return child };
    BUILD.with(|slot| {
        let mut slot = slot.borrow_mut();
        let build = slot.as_mut().unwrap();
        assert_eq!(build.stack.pop(), Some(ticket.id));
        if build.stack.is_empty() {
            let mut state = build.state.lock().unwrap();
            let changed = !state.ready
                || state.roots != build.roots
                || state.nodes.len() != build.nodes.len()
                || build.nodes.iter().any(|(id, entry)| {
                    state
                        .nodes
                        .get(id)
                        .map_or(true, |previous| previous.node != entry.node)
                });
            state.nodes = std::mem::take(&mut build.nodes);
            state.keys = std::mem::take(&mut build.keys);
            state.roots = std::mem::take(&mut build.roots);
            state.ready = true;
            if changed {
                state.wake();
            }
        }
    });
    Element::new(SemanticWidget { ticket, child })
}
struct SemanticWidget<'a> {
    ticket: Ticket,
    child: Element<'a, IcedMessage>,
}
#[derive(Default)]
struct FocusState {
    node: u64,
    focused: bool,
}
impl iced::advanced::widget::operation::Focusable for FocusState {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}
struct ReadFocus {
    target: iced::widget::Id,
    focused: bool,
}
impl Operation for ReadFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn focusable(
        &mut self,
        id: Option<&iced::widget::Id>,
        _: Rectangle,
        state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        if id == Some(&self.target) {
            self.focused = state.is_focused();
        }
    }
}
impl SemanticWidget<'_> {
    fn read_input_focus(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        if !self.ticket.button_focus {
            if let Some(target) = self.ticket.focus.clone() {
                let mut probe = ReadFocus {
                    target,
                    focused: false,
                };
                self.child.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut probe,
                );
                self.note_focus(probe.focused);
            }
        }
    }
    fn note_focus(&self, focused: bool) {
        let mut state = self.ticket.state.lock().unwrap();
        if focused && state.focus != NodeId(self.ticket.id) {
            state.focus = NodeId(self.ticket.id);
            state.wake();
        } else if !focused && state.focus == NodeId(self.ticket.id) {
            state.focus = ROOT;
            state.wake();
        }
    }
}
impl Widget<IcedMessage, Theme, iced::Renderer> for SemanticWidget<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FocusState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(FocusState {
            node: self.ticket.id,
            focused: false,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }
    fn diff(&self, tree: &mut Tree) {
        let focus = tree.state.downcast_mut::<FocusState>();
        if focus.node != self.ticket.id {
            // Positional iced slots can survive a keyed reorder/remount. Keep
            // their interaction state from becoming the new owner's state.
            *focus = FocusState {
                node: self.ticket.id,
                focused: false,
            };
            tree.children = vec![Tree::new(&self.child)];
            let mut state = self.ticket.state.lock().unwrap();
            if state.focus == NodeId(self.ticket.id) && self.ticket.focus.is_some() {
                // Reacquire the still-live owner's focus through the normal
                // operation after layout, with the same stale-action guard.
                if state.actions.len() < QUEUE_CAP {
                    state.actions.push_back(Envelope {
                        node: self.ticket.id,
                        action: Requested::Focus,
                    });
                    state.wake();
                }
            }
        } else {
            tree.diff_children(std::slice::from_ref(&self.child));
        }
    }
    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.child.as_widget().size_hint()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.child
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, IcedMessage>,
        viewport: &Rectangle,
    ) {
        if self.ticket.button_focus && tree.state.downcast_ref::<FocusState>().focused {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) = event {
                if matches!(
                    key,
                    iced::keyboard::Key::Named(
                        iced::keyboard::key::Named::Enter | iced::keyboard::key::Named::Space
                    )
                ) {
                    if let Some(message) = &self.ticket.click {
                        shell.publish(message.clone());
                        shell.capture_event();
                        return;
                    }
                }
            }
        }
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        self.read_input_focus(tree, layout, renderer);
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        {
            let mut state = self.ticket.state.lock().unwrap();
            let bounds = layout.bounds().intersection(viewport);
            if let Some(entry) = state.nodes.get_mut(&self.ticket.id) {
                if entry.bounds != bounds {
                    entry.bounds = bounds;
                    state.wake();
                }
            }
        }
        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.ticket.button_focus {
            if let Some(id) = &self.ticket.focus {
                let focus = tree.state.downcast_mut::<FocusState>();
                operation.focusable(Some(id), layout.bounds(), focus);
                self.note_focus(focus.focused);
            }
        }
        self.child
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
        self.read_input_focus(tree, layout, renderer);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, IcedMessage, Theme, iced::Renderer>> {
        self.child.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        convert_view_messages,
        devtools::{
            input_cache_key, patch_input_values_for_component, retain_input_values_after_handler,
            track_input_text,
        },
    };
    use super::*;

    fn fixture() -> DynamicComponent {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plan068_accessibility/app.at");
        crate::build_dynamic_component(&std::fs::read_to_string(&path).unwrap(), path.to_str())
            .unwrap()
    }
    fn render(
        window: iced::window::Id,
        component: &mut DynamicComponent,
        cache: &HashMap<String, String>,
    ) {
        render_with_capture(window, component, cache, false);
    }
    fn render_with_capture(
        window: iced::window::Id,
        component: &mut DynamicComponent,
        cache: &HashMap<String, String>,
        capture: bool,
    ) {
        prepare(window);
        let (mut view, _, _) = component.view_with_debug_gated(capture);
        patch_input_values_for_component(&mut view, cache, component);
        let element = with_scope(window, component, || {
            super::super::devtools::render_dynamic_view(
                convert_view_messages(view),
                None,
                &mut vec![],
            )
        });
        let mut ui = iced_test::simulator(element);
        // Exercise actual layout/draw. These are mechanism tests, not OS-tree evidence.
        let _snapshot = ui.snapshot(&Theme::Dark).expect("headless production draw");
    }
    fn find(window: iced::window::Id, identity: &str, role: Role) -> u64 {
        let state = shared(window);
        let state = state.lock().unwrap();
        let found = state
            .nodes
            .iter()
            .filter(|(_, entry)| {
                entry.owner.identity.as_deref() == Some(identity) && entry.node.role() == role
            })
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1, "one target for {identity}: {found:?}");
        found[0]
    }
    fn request(node: u64, action: Requested) -> IcedMessage {
        IcedMessage {
            widget: String::new(),
            event: ACTION.into(),
            input_value: Some(serde_json::to_string(&Envelope { node, action }).unwrap()),
        }
    }
    fn apply(
        window: iced::window::Id,
        component: &mut DynamicComponent,
        node: u64,
        action: Requested,
    ) {
        let Some(Dispatch::Message(message)) = resolve(window, component, &request(node, action))
        else {
            panic!("expected an accepted production message")
        };
        component.on_with_input_for(&message.widget, &message.event, message.input_value);
    }
    const RESULT_LAYOUT_TEXT: &str = "w03-cover.epub：书库中已存在\n笔记10.md：书库中已存在\n坏书.epub：导入失败：缺少 META-INF/container.xml（不是合法 EPUB 容器）\n中文 书2.txt：书库中已存在";

    #[test]
    fn capped_scroll_result_text_has_a_visible_measured_rectangle() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plan068_result_layout.at");
        let source = std::fs::read_to_string(path).unwrap();
        for root_style in [
            "w-full h-full p-6 gap-5",
            "p-6 md:p-8 gap-5 max-w-6xl mx-auto w-full",
        ] {
            for capture in [false, true] {
                let source = source.replace("w-full h-full p-6 gap-5", root_style);
                let mut component = crate::build_dynamic_component(&source, None).unwrap();
                let window = iced::window::Id::unique();
                render_with_capture(window, &mut component, &HashMap::new(), capture);
                let node = *shared(window)
                    .lock()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, entry)| entry.node.label() == Some("Fill result"))
                    .unwrap()
                    .0;
                apply(window, &mut component, node, Requested::Click);
                render_with_capture(window, &mut component, &HashMap::new(), capture);
                let state = shared(window);
                let state = state.lock().unwrap();
                let result = state
                    .nodes
                    .values()
                    .find(|entry| entry.node.value() == Some(RESULT_LAYOUT_TEXT))
                    .expect("the production converted tree must contain the result");
                assert!(result.bounds.is_some_and(|bounds| bounds.width >= 100.0 && bounds.height >= 40.0),
                "four result lines must have visible bounds under {root_style}, capture={capture}, got {:?}", result.bounds);
                drop(state);
                remove(window);
            }
        }
    }
    #[test]
    fn capped_result_keeps_intrinsic_height_before_tall_library_content() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plan068_result_layout.at");
        let fixture = std::fs::read_to_string(path)
            .unwrap()
            .replace(
                "w-full h-full p-6 gap-5",
                "p-6 md:p-8 gap-5 max-w-6xl mx-auto w-full",
            )
            .replace(
                "text \"Next controls\"",
                "col {\n style: \"h-[800px] shrink-0\"\n text \"Tall library content\"\n }",
            );
        for result_style in [
            "p-3 rounded-lg bg-muted text-sm text-foreground max-h-32 overflow-y-auto shrink-0",
            "p-3 text-sm text-foreground max-h-32 overflow-y-auto shrink-0",
            "h-full p-3 rounded-lg bg-muted text-sm text-foreground max-h-32 overflow-y-auto shrink-0",
        ] {
        for capture in [false, true] {
        let source = fixture.replace(
            "p-3 rounded-lg bg-muted text-sm text-foreground max-h-32 overflow-y-auto shrink-0", result_style);
        let mut component = crate::build_dynamic_component(&source, None).unwrap();
        let window = iced::window::Id::unique();
        render_with_capture(window, &mut component, &HashMap::new(), capture);
        let node = *shared(window).lock().unwrap().nodes.iter()
            .find(|(_, entry)| entry.node.label() == Some("Fill result")).unwrap().0;
        apply(window, &mut component, node, Requested::Click);
        render_with_capture(window, &mut component, &HashMap::new(), capture);
        let state = shared(window);
        let state = state.lock().unwrap();
        let result = state.nodes.values()
            .find(|entry| entry.node.value() == Some(RESULT_LAYOUT_TEXT))
            .expect("result text remains in the production tree");
        assert!(result.bounds.is_some_and(|bounds| bounds.width >= 100.0 && bounds.height >= 40.0),
            "a capped result must keep visible intrinsic height before overflowing siblings, {result_style}, capture={capture}: {:?}", result.bounds);
        drop(state);
        remove(window);
        }
        }
    }
    #[test]
    fn capped_child_result_remains_measurable_through_mcp_session_projection() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plan068_result_layout.at");
        let source = std::fs::read_to_string(path).unwrap().replace(
            "w-full h-full p-6 gap-5",
            "p-6 md:p-8 gap-5 max-w-6xl mx-auto w-full",
        );
        let component = crate::build_dynamic_component(&source, None).unwrap();
        let mcp = Arc::new(Mutex::new(crate::ui::mcp_server::SharedState::new(
            "LayoutApp".into(),
        )));
        let mut session = crate::ui::session::DesktopSession::empty(Some(mcp));
        let app = session.allocate_app(component);
        let window = iced::window::Id::unique();
        session.register_window(window, app, iced::Size::new(1200., 800.));
        prepare(window);
        {
            let element = super::super::projection::dynamic_view(
                session.split_ref_at(app, window).unwrap(),
                true,
            );
            let mut ui = iced_test::simulator(element);
            let _ = ui.snapshot(&Theme::Light).unwrap();
        }
        let (node, identity) = {
            let state = shared(window);
            let state = state.lock().unwrap();
            let (node, entry) = state
                .nodes
                .iter()
                .find(|(_, entry)| entry.node.label() == Some("Fill result"))
                .unwrap();
            (*node, entry.owner.identity.clone().unwrap())
        };
        apply(
            window,
            &mut session.apps.get_mut(&app).unwrap().component,
            node,
            Requested::Click,
        );
        // The host update tail transfers this dirty bit to its cached-view gate.
        assert!(session.apps[&app].component.is_dirty());
        *session.apps[&app].state.view_dirty.borrow_mut() = true;
        {
            let element = super::super::projection::dynamic_view(
                session.split_ref_at(app, window).unwrap(),
                true,
            );
            let mut ui = iced_test::simulator(element);
            let _ = ui.snapshot(&Theme::Light).unwrap();
        }
        assert_eq!(
            session.apps[&app]
                .component
                .bridge()
                .read_instance_state(&identity, "result")
                .unwrap()
                .as_str(),
            RESULT_LAYOUT_TEXT
        );
        let state = shared(window);
        let state = state.lock().unwrap();
        let result = state
            .nodes
            .values()
            .find(|entry| entry.node.value() == Some(RESULT_LAYOUT_TEXT))
            .expect("the MCP-enabled session must render the actual child result");
        assert!(
            result
                .bounds
                .is_some_and(|bounds| bounds.width >= 100.0 && bounds.height >= 40.0),
            "MCP capture must not hide the visible result, got {:?}",
            result.bounds
        );
        drop(state);
        remove(window);
    }
    #[test]
    fn unchanged_production_frames_do_not_schedule_another_accessibility_tick() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        let state = shared(window);
        state.lock().unwrap().wake_pending = false; // the preceding tick was consumed
        render(window, &mut component, &HashMap::new());
        assert!(
            !state.lock().unwrap().wake_pending,
            "unchanged layout and semantics must not sustain a redraw/wake loop"
        );
        let a = find(window, "/Probe#a", Role::TextInput);
        apply(
            window,
            &mut component,
            a,
            Requested::Value("changed semantics".into()),
        );
        render(window, &mut component, &HashMap::new());
        assert!(
            state.lock().unwrap().wake_pending,
            "changed values must publish"
        );
        state.lock().unwrap().wake_pending = false;
        render(window, &mut component, &HashMap::new());
        assert!(
            !state.lock().unwrap().wake_pending,
            "the changed frame must settle"
        );
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        render(window, &mut component, &HashMap::new());
        assert!(
            state.lock().unwrap().wake_pending,
            "removing live controls must publish a changed tree"
        );
        remove(window);
    }
    #[test]
    fn measured_controls_dispatch_to_one_original_instance_and_scale_bounds() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        let a = find(window, "/Probe#a", Role::Button);
        let b = find(window, "/Probe#b", Role::Button);
        {
            let state = shared(window);
            let mut state = state.lock().unwrap();
            let a_bounds = state.nodes[&a].bounds.unwrap();
            let b_bounds = state.nodes[&b].bounds.unwrap();
            assert!(a_bounds.width > 0. && a_bounds.height > 0. && b_bounds.y > a_bounds.y);
            state.scale = 1.5;
            let tree = state.snapshot().unwrap();
            let bounds = tree
                .nodes
                .iter()
                .find(|(id, _)| id.0 == a)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert_eq!(bounds.x0, a_bounds.x as f64 * 1.5);
            assert_eq!(bounds.y1, (a_bounds.y + a_bounds.height) as f64 * 1.5);
        }
        apply(window, &mut component, a, Requested::Click);
        assert_eq!(
            component
                .bridge()
                .read_instance_state("/Probe#a", "count")
                .unwrap(),
            auto_val::Value::Int(1)
        );
        assert_eq!(
            component
                .bridge()
                .read_instance_state("/Probe#b", "count")
                .unwrap(),
            auto_val::Value::Int(0)
        );
        remove(window);
    }
    #[test]
    fn unmount_remount_rejects_old_node_before_and_after_rebuild() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        let old = find(window, "/Probe#b", Role::Button);
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        let _ = component.view_with_debug_gated(false);
        assert!(matches!(
            resolve(window, &component, &request(old, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        render(window, &mut component, &HashMap::new());
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        render(window, &mut component, &HashMap::new());
        let fresh = find(window, "/Probe#b", Role::Button);
        assert_ne!(old, fresh);
        assert!(matches!(
            resolve(window, &component, &request(old, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        apply(window, &mut component, fresh, Requested::Click);
        assert_eq!(
            component
                .bridge()
                .read_instance_state("/Probe#b", "count")
                .unwrap(),
            auto_val::Value::Int(1)
        );
        remove(window);
    }
    #[test]
    fn explicit_released_owner_never_rebinds_to_the_remaining_same_type() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        let _ = component.view_with_debug_gated(false);
        assert!(component
            .bridge()
            .child_instance_state_id("/Probe#b")
            .is_none());

        // A delayed converted view still carries B's explicit identity. Only A
        // remains live; a unique-type fallback must not retarget this action.
        let stale_view = AbstractView::Button {
            label: "Released second".into(),
            onclick: IcedMessage {
                widget: "Probe\u{1}/Probe#b".into(),
                event: "Inc".into(),
                input_value: None,
            },
            style: None,
            on_right_click: None,
            content: None,
            disabled: false,
        };
        let element = with_scope(window, &component, || {
            super::super::devtools::render_dynamic_view(stale_view, None, &mut vec![])
        });
        let mut ui = iced_test::simulator(element);
        let _ = ui.snapshot(&Theme::Dark).unwrap();
        let node = *shared(window)
            .lock()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, entry)| entry.node.role() == Role::Button)
            .unwrap()
            .0;
        assert!(
            matches!(
                resolve(window, &component, &request(node, Requested::Click)),
                Some(Dispatch::Rejected)
            ),
            "a released explicit B must not become A"
        );
        assert!(
            matches!(
                resolve(window, &component, &request(node, Requested::Focus)),
                Some(Dispatch::Rejected)
            ),
            "an unresolved owner must not acquire focus"
        );

        // The remaining live instance still has an accepted ordinary control.
        render(window, &mut component, &HashMap::new());
        let a = find(window, "/Probe#a", Role::Button);
        apply(window, &mut component, a, Requested::Click);
        assert_eq!(
            component
                .bridge()
                .read_instance_state("/Probe#a", "count")
                .unwrap(),
            auto_val::Value::Int(1)
        );
        remove(window);
    }
    #[test]
    fn typed_text_cache_is_instance_and_generation_bound() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        let mut cache = HashMap::new();
        render(window, &mut component, &cache);
        let a = find(window, "/Probe#a", Role::TextInput);
        let message = match resolve(
            window,
            &component,
            &request(a, Requested::Value("甲中文\\路径".into())),
        )
        .unwrap()
        {
            Dispatch::Message(message) => message,
            _ => panic!("value message"),
        };
        let key = input_cache_key(&component, &message.widget, "Edit").unwrap();
        track_input_text(&mut cache, &key, message.input_value.as_deref().unwrap());
        component.on_with_input_for(&message.widget, &message.event, message.input_value);
        retain_input_values_after_handler(&mut cache, &component, &key);
        render(window, &mut component, &cache);
        let b = find(window, "/Probe#b", Role::TextInput);
        {
            let state = shared(window);
            let state = state.lock().unwrap();
            assert_eq!(state.nodes[&a].node.value(), Some("甲中文\\路径"));
            assert_eq!(state.nodes[&b].node.value(), Some("initial"));
        }
        // A constant value still needs its own cache. It must never patch B.
        assert_ne!(
            key,
            input_cache_key(&component, "Probe\u{1}/Probe#b", "Edit").unwrap()
        );
        let stale = input_cache_key(&component, "Probe\u{1}/Probe#b", "Edit").unwrap();
        cache.insert(stale.clone(), "stale B".into());
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        let _ = component.view_with_debug_gated(false);
        retain_input_values_after_handler(&mut cache, &component, "ToggleB");
        assert!(!cache.contains_key(&stale));
        component
            .call_widget_handler("P068Accessibility", "ToggleB", &[])
            .unwrap();
        render(window, &mut component, &cache);
        let fresh = find(window, "/Probe#b", Role::TextInput);
        assert_ne!(fresh, b);
        {
            let state = shared(window);
            assert_eq!(
                state.lock().unwrap().nodes[&fresh].node.value(),
                Some("initial")
            );
        }
        remove(window);
    }
    #[test]
    fn another_app_cannot_consume_a_windows_system_action() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        let other = fixture();
        render(window, &mut component, &HashMap::new());
        let a = find(window, "/Probe#a", Role::Button);
        assert!(matches!(
            resolve(window, &other, &request(a, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        assert_eq!(
            other.bridge().read_state("show_b").unwrap(),
            auto_val::Value::Bool(true)
        );
        remove(window);
    }
    #[test]
    fn list_children_publish_selection_and_disabled_controls_reject_actions() {
        let window = iced::window::Id::unique();
        let component = fixture();
        prepare(window);
        let message = IcedMessage {
            widget: "P068Accessibility".into(),
            event: "ToggleB".into(),
            input_value: None,
        };
        let view = AbstractView::List {
            items: vec![
                AbstractView::Checkbox {
                    is_checked: true,
                    label: "Selected book".into(),
                    on_toggle: Some(message.clone()),
                    style: None,
                },
                AbstractView::Radio {
                    is_selected: true,
                    label: "Selected sort".into(),
                    on_select: Some(message.clone()),
                    style: None,
                },
                AbstractView::Button {
                    label: "Disabled action".into(),
                    onclick: message,
                    style: None,
                    on_right_click: None,
                    content: None,
                    disabled: true,
                },
            ],
            spacing: 8,
            style: None,
        };
        let element = with_scope(window, &component, || {
            super::super::devtools::render_dynamic_view(view, None, &mut vec![])
        });
        let mut ui = iced_test::simulator(element);
        let _ = ui.snapshot(&Theme::Dark).unwrap();
        let disabled = {
            let state = shared(window);
            let state = state.lock().unwrap();
            let list = state
                .nodes
                .values()
                .find(|entry| entry.node.role() == Role::List)
                .unwrap();
            assert_eq!(list.node.children().len(), 3);
            let checkbox = state
                .nodes
                .values()
                .find(|entry| entry.node.role() == Role::CheckBox)
                .unwrap();
            assert_eq!(checkbox.node.toggled(), Some(Toggled::True));
            let radio = state
                .nodes
                .values()
                .find(|entry| entry.node.role() == Role::RadioButton)
                .unwrap();
            assert_eq!(radio.node.is_selected(), Some(true));
            *state
                .nodes
                .iter()
                .find(|(_, entry)| entry.node.is_disabled())
                .unwrap()
                .0
        };
        assert!(matches!(
            resolve(window, &component, &request(disabled, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        assert!(matches!(
            resolve(window, &component, &request(disabled, Requested::Focus)),
            Some(Dispatch::Rejected)
        ));
        assert!(matches!(
            resolve(window, &component, &request(u64::MAX, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        remove(window);
    }

    #[test]
    fn reused_iced_slot_does_not_transfer_button_focus_to_a_different_owner() {
        use iced::advanced::widget::operation::Focusable;
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        let a = find(window, "/Probe#a", Role::Button);
        let b = find(window, "/Probe#b", Role::Button);
        let state = shared(window);
        let ticket = |id| {
            let entry = state.lock().unwrap().nodes[&id].clone();
            Ticket {
                state: state.clone(),
                id,
                focus: entry.focus,
                click: entry.click,
                button_focus: true,
            }
        };
        let old = SemanticWidget {
            ticket: ticket(a),
            child: iced::widget::text("First action").into(),
        };
        let mut tree = Tree::new(&old as &dyn Widget<IcedMessage, Theme, iced::Renderer>);
        tree.state.downcast_mut::<FocusState>().focus();
        let next = SemanticWidget {
            ticket: ticket(b),
            child: iced::widget::text("Second action").into(),
        };
        next.diff(&mut tree);
        assert_eq!(tree.state.downcast_ref::<FocusState>().node, b);
        assert!(!tree.state.downcast_ref::<FocusState>().is_focused());
        assert!(
            state.lock().unwrap().actions.is_empty(),
            "a new owner must not receive a focus request from the old slot"
        );
        remove(window);
    }
    #[test]
    fn first_frame_child_controls_are_bound_without_debug_or_a_prerender() {
        let window = iced::window::Id::unique();
        let mut session = crate::ui::session::DesktopSession::empty(None);
        let app = session.allocate_app(fixture());
        session.register_window(window, app, iced::Size::new(800., 600.));
        prepare(window);
        assert!(
            session.apps[&app]
                .component
                .bridge()
                .child_instance_identities()
                .is_empty(),
            "the accepted first-frame control must not pre-mount children"
        );
        {
            let view = session.split_ref_at(app, window).unwrap();
            let element = super::super::projection::dynamic_view(view, false);
            let mut ui = iced_test::simulator(element);
            let _ = ui.snapshot(&Theme::Dark).unwrap();
        }
        let a = find(window, "/Probe#a", Role::TextInput);
        apply(
            window,
            &mut session.apps.get_mut(&app).unwrap().component,
            a,
            Requested::Value("first frame".into()),
        );
        assert_eq!(
            session.apps[&app]
                .component
                .bridge()
                .read_instance_state("/Probe#a", "text_value")
                .unwrap()
                .as_str(),
            "first frame"
        );
        assert_eq!(
            session.apps[&app]
                .component
                .bridge()
                .read_instance_state("/Probe#b", "text_value")
                .unwrap()
                .as_str(),
            "initial"
        );
        remove(window);
    }
    #[test]
    fn closing_a_window_discards_its_actions_without_recreating_the_registry() {
        let window = iced::window::Id::unique();
        let mut component = fixture();
        render(window, &mut component, &HashMap::new());
        let a = find(window, "/Probe#a", Role::Button);
        remove(window);
        assert!(matches!(
            resolve(window, &component, &request(a, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        assert!(!windows().lock().unwrap().contains_key(&window));
        let fresh = iced::window::Id::unique();
        let mut other = fixture();
        render(fresh, &mut other, &HashMap::new());
        assert!(matches!(
            resolve(window, &other, &request(a, Requested::Click)),
            Some(Dispatch::Rejected)
        ));
        assert!(!windows().lock().unwrap().contains_key(&window));
        remove(fresh);
    }
    #[test]
    fn actual_routed_iced_button_carrier_writes_the_pages_own_model() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plan068_routes/src/front/app.at");
        let mut component =
            crate::build_dynamic_component(&std::fs::read_to_string(&path).unwrap(), path.to_str())
                .unwrap();
        component.set_route("/");
        let window = iced::window::Id::unique();
        let mut session = crate::ui::session::DesktopSession::empty(None);
        let app = session.allocate_app(component);
        session.register_window(window, app, iced::Size::new(800., 600.));
        prepare(window);
        {
            let element = super::super::projection::dynamic_view(
                session.split_ref_at(app, window).unwrap(),
                false,
            );
            let mut ui = iced_test::simulator(element);
            let _ = ui.snapshot(&Theme::Dark).unwrap();
        }
        let (node, identity) = {
            let state = shared(window);
            let state = state.lock().unwrap();
            let (id, entry) = state
                .nodes
                .iter()
                .find(|(_, entry)| entry.node.label() == Some("Write local"))
                .unwrap();
            let identity = entry.owner.identity.clone().unwrap();
            assert!(identity.contains('@'));
            (*id, identity)
        };
        let component = &mut session.apps.get_mut(&app).unwrap().component;
        apply(window, component, node, Requested::Click);
        assert_eq!(
            component
                .bridge()
                .read_instance_state(&identity, "result")
                .unwrap()
                .as_str(),
            "local result"
        );
        assert_eq!(
            component.bridge().read_state("result").unwrap().as_str(),
            ""
        );
        remove(window);
    }
}
