//! Keyboard focus for action controls. The inner button/checkbox retains its
//! ordinary mouse handler; Enter/Space dispatch that exact same message.
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::Renderer as _;
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};

pub fn action_key(widget: &str, event: &str) -> String {
    format!("action_focus:{widget}:{event}")
}

pub fn control_id(widget: &str, event: &str, focus_key: Option<&str>) -> iced::widget::Id {
    iced::widget::Id::from(match focus_key.filter(|key| !key.is_empty()) {
        Some(key) => key.to_owned(),
        None => action_key(widget, event),
    })
}

pub struct State<M> {
    pub focused: bool,
    id: iced::widget::Id,
    activation: Option<M>,
    key_bindings: Vec<(String, M)>,
    retain: bool,
}
impl<M> widget::operation::Focusable for State<M> {
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

pub struct FocusTarget<'a, Message> {
    content: Element<'a, Message>,
    id: iced::widget::Id,
    activation: Option<Message>,
    key_bindings: Vec<(String, Message)>,
    retain: bool,
}
impl<'a, Message> FocusTarget<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        id: iced::widget::Id,
        activation: Message,
    ) -> Self {
        Self {
            content: content.into(),
            id,
            activation: Some(activation),
            key_bindings: Vec::new(),
            retain: false,
        }
    }

    pub fn key_bindings(mut self, bindings: Vec<(String, Message)>, retain: bool) -> Self {
        self.key_bindings = bindings;
        self.retain = retain;
        self
    }

    pub fn group(content: Element<'a, Message>, id: iced::widget::Id) -> Self {
        Self { content, id, activation: None, key_bindings: Vec::new(), retain: false }
    }
}

fn retained_focus<M: 'static>(tree: &Tree) -> Option<iced::widget::Id> {
    if tree.tag == widget::tree::Tag::of::<State<M>>() {
        let state = tree.state.downcast_ref::<State<M>>();
        if state.retain && state.focused {
            return Some(state.id.clone());
        }
    }
    tree.children.iter().find_map(retained_focus::<M>)
}

fn restore_focus<M: 'static>(tree: &mut Tree, target: Option<&iced::widget::Id>) -> bool {
    if tree.tag == widget::tree::Tag::of::<State<M>>() {
        let state = tree.state.downcast_mut::<State<M>>();
        if state.retain && target.is_none_or(|id| &state.id == id) {
            state.focused = true;
            return true;
        }
    }
    tree.children.iter_mut().any(|child| restore_focus::<M>(child, target))
}

fn scoped_key_name(key: &iced::keyboard::Key) -> Option<&str> {
    use iced::keyboard::{key::Named, Key};
    match key {
        Key::Named(Named::ArrowLeft) => Some("left"),
        Key::Named(Named::ArrowRight) => Some("right"),
        Key::Named(Named::ArrowUp) => Some("up"),
        Key::Named(Named::ArrowDown) => Some("down"),
        Key::Named(Named::Home) => Some("home"),
        Key::Named(Named::End) => Some("end"),
        Key::Named(Named::Enter) => Some("enter"),
        Key::Named(Named::Space) => Some("space"),
        Key::Character(text) if text.as_str() == " " => Some("space"),
        _ => None,
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for FocusTarget<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State<Message>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            focused: false,
            id: self.id.clone(),
            activation: self.activation.clone(),
            key_bindings: self.key_bindings.clone(),
            retain: self.retain,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let target = self.activation.is_none().then(|| retained_focus::<Message>(&tree.children[0])).flatten();
        let state = tree.state.downcast_mut::<State<Message>>();
        if state.id != self.id {
            state.focused = false;
        }
        state.id = self.id.clone();
        state.activation = self.activation.clone();
        state.key_bindings.clone_from(&self.key_bindings);
        state.retain = self.retain;
        tree.diff_children(&[self.content.as_widget()]);
        if let Some(target) = target.as_ref() {
            let restored = restore_focus::<Message>(&mut tree.children[0], Some(target))
                || restore_focus::<Message>(&mut tree.children[0], None);
            tree.state.downcast_mut::<State<Message>>().focused = !restored;
        }
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Message>>();
        if self.activation.is_some() && matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            state.focused = cursor.is_over(layout.bounds());
        }
        let scoped_message = match event {
            Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. })
                if state.focused && modifiers.is_empty() =>
            {
                scoped_key_name(key).and_then(|name| {
                    self.key_bindings.iter().find(|(key, _)| key == name)
                })
            }
            _ => None,
        };
        if let Some((_, message)) = scoped_message {
            shell.publish(message.clone());
            shell.capture_event();
            return;
        }
        if state.focused
            && matches!(event, Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. })
            if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter | iced::keyboard::key::Named::Space)) || matches!(key, iced::keyboard::Key::Character(text) if text.as_str() == " "))
        {
            if let Some(message) = &self.activation {
                shell.publish(message.clone());
                shell.capture_event();
                return;
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
        if tree.state.downcast_ref::<State<Message>>().focused {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: inherited_style.text_color,
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Message>>();
        if self.activation.is_some() || state.focused {
            operation.focusable(Some(&self.id), layout.bounds(), state);
            operation.custom(Some(&self.id), layout.bounds(), state);
        }
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<FocusTarget<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(widget: FocusTarget<'a, Message>) -> Self {
        Self::new(widget)
    }
}

/// The same focused action is used by the keyboard automation channel. This
/// operation traverses native widget state, rather than guessing a first row.
pub struct ActivateFocused<M> {
    result: Option<M>,
    focused: Option<iced::widget::Id>,
    key: Option<String>,
}
impl<M> Default for ActivateFocused<M> {
    fn default() -> Self {
        Self {
            result: None,
            focused: None,
            key: None,
        }
    }
}
impl<M> ActivateFocused<M> {
    pub fn for_key(key: String) -> Self {
        Self {
            key: Some(key),
            ..Self::default()
        }
    }
}
impl<M: Clone + Send + 'static> widget::Operation<(Option<iced::widget::Id>, Option<M>)>
    for ActivateFocused<M>
{
    fn focusable(
        &mut self,
        id: Option<&iced::widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if state.is_focused() {
            self.focused = id.cloned();
        }
    }
    fn custom(
        &mut self,
        _id: Option<&iced::widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if let Some(state) = state.downcast_ref::<State<M>>() {
            if state.focused {
                self.result = if let Some(key) = &self.key {
                    state
                        .key_bindings
                        .iter()
                        .find_map(|(name, msg)| (name == key).then(|| msg.clone()))
                } else {
                    state.activation.clone()
                };
            }
        }
    }
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn widget::Operation<(Option<iced::widget::Id>, Option<M>)>),
    ) {
        operate(self);
    }
    fn finish(&self) -> widget::operation::Outcome<(Option<iced::widget::Id>, Option<M>)> {
        widget::operation::Outcome::Some((self.focused.clone(), self.result.clone()))
    }
}

/// Focus effects are checked when the native widget tree is visited, so an
/// operation queued by an old modal session cannot steal a reopened scope.
pub fn guarded_focus<T>(
    target: iced::widget::Id,
    generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
    expected: u64,
) -> impl widget::Operation<T> {
    struct Focus {
        target: iced::widget::Id,
        generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
        expected: u64,
    }
    impl<T> widget::Operation<T> for Focus {
        fn focusable(
            &mut self,
            id: Option<&iced::widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn widget::operation::Focusable,
        ) {
            if self.generation.load(std::sync::atomic::Ordering::Acquire) != self.expected {
                return;
            }
            if id == Some(&self.target) {
                state.focus();
            } else {
                state.unfocus();
            }
        }
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation<T>)) {
            operate(self);
        }
    }
    Focus {
        target,
        generation,
        expected,
    }
}

pub fn guarded_select_all<T>(
    target: iced::widget::Id,
    generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
    expected: u64,
) -> impl widget::Operation<T> {
    struct SelectAll {
        target: iced::widget::Id,
        generation: std::sync::Arc<std::sync::atomic::AtomicU64>,
        expected: u64,
    }
    impl<T> widget::Operation<T> for SelectAll {
        fn text_input(
            &mut self,
            id: Option<&iced::widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn widget::operation::TextInput,
        ) {
            if id == Some(&self.target)
                && self.generation.load(std::sync::atomic::Ordering::Acquire) == self.expected
            {
                state.select_all();
            }
        }
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation<T>)) {
            operate(self);
        }
    }
    SelectAll { target, generation, expected }
}
