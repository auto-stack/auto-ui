//! A container for capturing mouse events.
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::{Operation, Tree, tree};
use crate::core::{
    Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell, Size,
    Vector, Widget,
};

/// Emit messages on mouse events.
pub struct MouseArea<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
    on_release: Option<Message>,
    on_double_click: Option<Message>,
    on_right_press: Option<Message>,
    on_right_release: Option<Message>,
    on_middle_press: Option<Message>,
    on_middle_release: Option<Message>,
    on_scroll: Option<Box<dyn Fn(mouse::ScrollDelta) -> Message + 'a>>,
    on_enter: Option<Message>,
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_exit: Option<Message>,
    interaction: Option<mouse::Interaction>,
    left_press_time: Option<PressClock>,
}

type PressTime = (u32, u32);
type PressClock = fn() -> Option<PressTime>;

thread_local! {
    static LEFT_PRESS_TIME: Cell<Option<PressTime>> = const { Cell::new(None) };
}

struct PressTimeScope(Option<PressTime>);

impl Drop for PressTimeScope {
    fn drop(&mut self) {
        LEFT_PRESS_TIME.set(self.0);
    }
}

fn is_left_press(event: &Event) -> bool {
    matches!(event, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)))
}

impl<'a, Message, Theme, Renderer> MouseArea<'a, Message, Theme, Renderer> {
    /// The message to emit on a left button press.
    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// The message to emit on a left button release.
    #[must_use]
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// The message to emit on a double click.
    ///
    /// If you use this with [`on_press`]/[`on_release`], those
    /// event will be emit as normal.
    ///
    /// The events stream will be: on_press -> on_release -> on_press
    /// -> on_double_click -> on_release -> on_press ...
    ///
    /// [`on_press`]: Self::on_press
    /// [`on_release`]: Self::on_release
    #[must_use]
    pub fn on_double_click(mut self, message: Message) -> Self {
        self.on_double_click = Some(message);
        self
    }

    /// Supply the native press time and double-click interval in milliseconds
    /// to the mouse areas inside this window's root.
    #[must_use]
    pub fn with_left_press_time(mut self, next: PressClock) -> Self {
        self.left_press_time = Some(next);
        self
    }

    /// The message to emit on a right button press.
    #[must_use]
    pub fn on_right_press(mut self, message: Message) -> Self {
        self.on_right_press = Some(message);
        self
    }

    /// The message to emit on a right button release.
    #[must_use]
    pub fn on_right_release(mut self, message: Message) -> Self {
        self.on_right_release = Some(message);
        self
    }

    /// The message to emit on a middle button press.
    #[must_use]
    pub fn on_middle_press(mut self, message: Message) -> Self {
        self.on_middle_press = Some(message);
        self
    }

    /// The message to emit on a middle button release.
    #[must_use]
    pub fn on_middle_release(mut self, message: Message) -> Self {
        self.on_middle_release = Some(message);
        self
    }

    /// The message to emit when scroll wheel is used
    #[must_use]
    pub fn on_scroll(
        mut self,
        on_scroll: impl Fn(mouse::ScrollDelta) -> Message + 'a,
    ) -> Self {
        self.on_scroll = Some(Box::new(on_scroll));
        self
    }

    /// The message to emit when the mouse enters the area.
    #[must_use]
    pub fn on_enter(mut self, message: Message) -> Self {
        self.on_enter = Some(message);
        self
    }

    /// The message to emit when the mouse moves in the area.
    #[must_use]
    pub fn on_move(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(on_move));
        self
    }

    /// The message to emit when the mouse exits the area.
    #[must_use]
    pub fn on_exit(mut self, message: Message) -> Self {
        self.on_exit = Some(message);
        self
    }

    /// The [`mouse::Interaction`] to use when hovering the area.
    #[must_use]
    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = Some(interaction);
        self
    }
}

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    is_hovered: bool,
    bounds: Rectangle,
    cursor_position: Option<Point>,
    previous_click: Option<mouse::Click>,
    previous_native_click: Option<(Point, u32)>,
    overlay_presses: VecDeque<Option<PressTime>>,
}

impl<'a, Message, Theme, Renderer> MouseArea<'a, Message, Theme, Renderer> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        MouseArea {
            content: content.into(),
            on_press: None,
            on_release: None,
            on_double_click: None,
            on_right_press: None,
            on_right_release: None,
            on_middle_press: None,
            on_middle_release: None,
            on_scroll: None,
            on_enter: None,
            on_move: None,
            on_exit: None,
            interaction: None,
            left_press_time: None,
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MouseArea<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            limits,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let _press_time = self.left_press_time.filter(|_| is_left_press(event)).map(|next| {
            let state = tree.state.downcast_mut::<State>();
            let time = state.overlay_presses.pop_front().unwrap_or_else(next);
            PressTimeScope(LEFT_PRESS_TIME.replace(time))
        });
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

        if !shell.is_event_captured() {
            update(self, tree, event, layout, cursor, shell);
        }

        // The child can own a single press while its area owns the double-click.
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) && let Some(position) = cursor.position_over(layout.bounds())
            && let Some(message) = self.on_double_click.as_ref()
        {
            let state = tree.state.downcast_mut::<State>();
            let double_click = if let Some((time, interval)) = LEFT_PRESS_TIME.get() {
                state.previous_click = None;
                let double_click = state.previous_native_click.is_some_and(|(previous, at)| {
                    previous.distance(position) < 6.0 && time.wrapping_sub(at) <= interval
                });
                state.previous_native_click = (!double_click).then_some((position, time));
                double_click
            } else {
                state.previous_native_click = None;
                let click = mouse::Click::new(position, mouse::Button::Left, state.previous_click);
                state.previous_click = Some(click);
                click.kind() == mouse::click::Kind::Double
            };
            if double_click {
                shell.publish(message.clone());
            }
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        match (self.interaction, content_interaction) {
            (Some(interaction), mouse::Interaction::None)
                if cursor.is_over(layout.bounds()) =>
            {
                interaction
            }
            _ => content_interaction,
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let content = self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )?;
        let Some(next) = self.left_press_time else { return Some(content) };
        let state = tree.state.downcast_mut::<State>();
        Some(overlay::Element::new(Box::new(PressTimeOverlay {
            content: RefCell::new(overlay::Nested::new(content)),
            next,
            forwarded: &mut state.overlay_presses,
        })))
    }
}

struct PressTimeOverlay<'a, Message, Theme, Renderer> {
    content: RefCell<overlay::Nested<'a, Message, Theme, Renderer>>,
    next: PressClock,
    forwarded: &'a mut VecDeque<Option<PressTime>>,
}

impl<Message, Theme, Renderer: renderer::Renderer> overlay::Overlay<Message, Theme, Renderer>
    for PressTimeOverlay<'_, Message, Theme, Renderer>
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.content.get_mut().layout(renderer, bounds)
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content.get_mut().operate(layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let press = is_left_press(event).then(self.next);
        let _scope = press.map(|time| PressTimeScope(LEFT_PRESS_TIME.replace(time)));
        self.content.get_mut().update(event, layout, cursor, renderer, clipboard, shell);
        // Iced processes the overlay's batch before dispatching its ignored
        // events to the base. Keep only those timestamps in that same order.
        if let Some(time) = press.filter(|_| !shell.is_event_captured()) {
            self.forwarded.push_back(time);
        }
    }

    fn mouse_interaction(
        &self, layout: Layout<'_>, cursor: mouse::Cursor, renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.borrow_mut().mouse_interaction(layout, cursor, renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.borrow_mut().draw(renderer, theme, style, layout, cursor);
    }
}

impl<'a, Message, Theme, Renderer> From<MouseArea<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(
        area: MouseArea<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(area)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`MouseArea`]
/// accordingly.
fn update<Message: Clone, Theme, Renderer>(
    widget: &mut MouseArea<'_, Message, Theme, Renderer>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let state: &mut State = tree.state.downcast_mut();

    let cursor_position = cursor.position();
    let bounds = layout.bounds();

    if state.cursor_position != cursor_position || state.bounds != bounds {
        let was_hovered = state.is_hovered;

        state.is_hovered = cursor.is_over(layout.bounds());
        state.cursor_position = cursor_position;
        state.bounds = bounds;

        match (
            widget.on_enter.as_ref(),
            widget.on_move.as_ref(),
            widget.on_exit.as_ref(),
        ) {
            (Some(on_enter), _, _) if state.is_hovered && !was_hovered => {
                shell.publish(on_enter.clone());
            }
            (_, Some(on_move), _) if state.is_hovered => {
                if let Some(position) = cursor.position_in(layout.bounds()) {
                    shell.publish(on_move(position));
                }
            }
            (_, _, Some(on_exit)) if !state.is_hovered && was_hovered => {
                shell.publish(on_exit.clone());
            }
            _ => {}
        }
    }

    if !cursor.is_over(layout.bounds()) {
        return;
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if let Some(message) = widget.on_press.as_ref() {
                shell.publish(message.clone());
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if let Some(message) = widget.on_release.as_ref() {
                shell.publish(message.clone());
            }
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
            if let Some(message) = widget.on_right_press.as_ref() {
                shell.publish(message.clone());
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
            if let Some(message) = widget.on_right_release.as_ref() {
                shell.publish(message.clone());
            }
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
            if let Some(message) = widget.on_middle_press.as_ref() {
                shell.publish(message.clone());
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
            if let Some(message) = widget.on_middle_release.as_ref() {
                shell.publish(message.clone());
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            if let Some(on_scroll) = widget.on_scroll.as_ref() {
                shell.publish(on_scroll(*delta));
                shell.capture_event();
            }
        }
        _ => {}
    }
}
