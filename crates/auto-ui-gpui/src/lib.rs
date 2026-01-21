// auto-ui-gpui: GPUI backend adapter for auto-ui
//
// This crate provides adapter traits to convert auto-ui's abstract View<M>
// into GPUI's render tree using gpui-component library.

use auto_ui::{View as AbstractView, Component};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, scroll::ScrollableElement, *};
use std::fmt::Debug;

/// Context for GPUI rendering with message passing
pub struct GpuiContext<M: Clone + Debug + 'static> {
    phantom: std::marker::PhantomData<M>,
}

impl<M: Clone + Debug + 'static> Default for GpuiContext<M> {
    fn default() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

/// Trait for converting abstract View<M> into GPUI element
///
/// This trait enables rendering auto-ui's abstract view tree using GPUI framework.
/// Unlike Iced, GPUI uses closures instead of message enums, so we need to handle
/// message conversion differently.
///
/// # Example
/// ```rust
/// use auto_ui::View;
/// use auto_ui_gpui::IntoGpuiElement;
///
/// let view: View<MyMessage> = View::button("Click", MyMessage::Click);
/// // The conversion requires a message handler callback
/// ```
pub trait IntoGpuiElement<M: Clone + Debug + 'static> {
    /// Convert abstract view into GPUI element with message handler
    fn into_gpui<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + 'static + Clone;

    /// Convert abstract view into GPUI element without message handler
    fn into_gpui_static(self) -> AnyElement
    where
        Self: Sized,
    {
        self.into_gpui(|_| {})
    }
}

impl<M: Clone + Debug + 'static> IntoGpuiElement<M> for AbstractView<M> {
    fn into_gpui<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + 'static + Clone,
    {
        // Use a counter for unique button IDs
        let mut button_counter = 0u64;

        match self {
            AbstractView::Empty => {
                // Empty view renders as empty div
                div().into_any()
            }

            AbstractView::Text(content) => {
                // Direct text rendering
                div().child(content).into_any()
            }

            AbstractView::Button { label, onclick: _ } => {
                // Button with click handler - note: we can't directly handle messages
                // in GPUI's Button without proper context. This is a simplified version.
                button_counter += 1;
                let label_clone = label.clone();
                Button::new(("button", button_counter))
                    .primary()
                    .label(label_clone)
                    .into_any_element()
            }

            AbstractView::Row { children, spacing, padding } => {
                let mut row_div = div().h_flex().gap(px(spacing as f32)).p(px(padding as f32));

                // Recursively convert children
                for child in children {
                    let handle_msg_clone = handle_msg.clone();
                    row_div = row_div.child(child.into_gpui(handle_msg_clone));
                }

                row_div.into_any()
            }

            AbstractView::Column { children, spacing, padding } => {
                let mut col_div = div().v_flex().gap(px(spacing as f32)).p(px(padding as f32));

                // Recursively convert children
                for child in children {
                    let handle_msg_clone = handle_msg.clone();
                    col_div = col_div.child(child.into_gpui(handle_msg_clone));
                }

                col_div.into_any()
            }

            AbstractView::Input {
                placeholder,
                value,
                on_change: _,
                width: _,
                password: _,
            } => {
                // GPUI text input
                // Note: gpui-component's Input implementation requires more complex setup
                // For now, we use a simple div that shows the value
                div()
                    .child(format!("{}: {}", placeholder, value))
                    .into_any()
            }

            AbstractView::Checkbox {
                is_checked,
                label,
                on_toggle: _,
            } => {
                // Checkbox with label - simplified version
                let display_text = format!(
                    "{} {}",
                    if is_checked { "[✓]" } else { "[ ]" },
                    label
                );

                div().child(display_text).into_any()
            }

            AbstractView::Container {
                child,
                padding,
                width,
                height,
                center_x,
                center_y,
            } => {
                let mut container_div = div().p(px(padding as f32));

                // Apply width
                if let Some(w) = width {
                    container_div = container_div.w(px(w as f32));
                }

                // Apply height
                if let Some(h) = height {
                    container_div = container_div.h(px(h as f32));
                }

                // Apply centering
                if center_x {
                    container_div = container_div.items_center();
                }
                if center_y {
                    container_div = container_div.justify_center();
                }

                // Add child
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.into_gpui(handle_msg_clone);
                container_div = container_div.child(child_element);

                container_div.into_any()
            }

            AbstractView::Scrollable {
                child,
                width,
                height,
            } => {
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.into_gpui(handle_msg_clone);

                let mut scroll_div = div().overflow_scrollbar();

                // Apply width
                if let Some(w) = width {
                    scroll_div = scroll_div.w(px(w as f32));
                }

                // Apply height
                if let Some(h) = height {
                    scroll_div = scroll_div.h(px(h as f32));
                }

                scroll_div.child(child_element).into_any_element()
            }

            AbstractView::Radio {
                label,
                is_selected,
                on_select: _,
            } => {
                // Radio button - simplified version
                let display_text = format!(
                    "{} {}",
                    if is_selected { "(•)" } else { "( )" },
                    label
                );

                div().child(display_text).into_any()
            }

            AbstractView::Select {
                options,
                selected_index,
                on_select: _,
            } => {
                // Select dropdown - simplified version
                let selected = selected_index
                    .and_then(|i| options.get(i).cloned())
                    .unwrap_or_default();

                div()
                    .child(format!("Select: {}", selected))
                    .into_any()
            }

            AbstractView::List { items, spacing } => {
                let mut list_div = div().v_flex().gap(px(spacing as f32));

                // Recursively convert items
                for item in items {
                    let handle_msg_clone = handle_msg.clone();
                    list_div = list_div.child(item.into_gpui(handle_msg_clone));
                }

                list_div.into_any()
            }

            AbstractView::Table {
                headers,
                rows,
                spacing,
                col_spacing,
            } => {
                let mut table_div = div().v_flex().gap(px(spacing as f32));

                // Add header row
                let mut header_row_div = div().h_flex().gap(px(col_spacing as f32));
                for header in headers {
                    let handle_msg_clone = handle_msg.clone();
                    header_row_div = header_row_div.child(header.into_gpui(handle_msg_clone));
                }
                table_div = table_div.child(header_row_div);

                // Add data rows
                for row_data in rows {
                    let mut row_div = div().h_flex().gap(px(col_spacing as f32));
                    for cell in row_data {
                        let handle_msg_clone = handle_msg.clone();
                        row_div = row_div.child(cell.into_gpui(handle_msg_clone));
                    }
                    table_div = table_div.child(row_div);
                }

                table_div.into_any()
            }
        }
    }
}

/// Extension trait for Component to add GPUI-compatible render method
///
/// This allows components to be used directly with GPUI's rendering system.
///
/// # Example
/// ```rust
/// use auto_ui::{Component, View};
/// use auto_ui_gpui::ComponentGpui;
///
/// struct MyComponent { ... }
///
/// impl Component for MyComponent {
///     type Msg = MyMessage;
///     fn on(&mut self, msg: Self::Msg) { ... }
///     fn view(&self) -> View<Self::Msg> { ... }
/// }
///
/// // Use with GPUI
/// let element = my_component.view_gpui();
/// ```
pub trait ComponentGpui: Component {
    /// GPUI-compatible view function with message handler
    fn view_gpui<F>(&self, handle_msg: F) -> AnyElement
    where
        Self::Msg: Clone + Debug + 'static,
        F: Fn(Self::Msg) + 'static + Clone,
    {
        self.view().into_gpui(handle_msg)
    }

    /// GPUI-compatible view function without message handler
    fn view_gpui_static(&self) -> AnyElement
    where
        Self::Msg: Clone + Debug + 'static,
    {
        self.view().into_gpui_static()
    }
}

// Blanket implementation for all Component types
impl<T: Component> ComponentGpui for T {}

/// Helper struct to bridge GPUI's listener pattern with auto-ui's message pattern
pub struct GpuiMessageBridge<C: Component> {
    component: C,
}

impl<C: Component> GpuiMessageBridge<C> {
    pub fn new(component: C) -> Self {
        Self { component }
    }

    pub fn update(&mut self, msg: C::Msg) {
        self.component.on(msg);
    }

    pub fn render<F>(&self, handle_msg: F) -> AnyElement
    where
        C::Msg: Clone + Debug + 'static,
        F: Fn(C::Msg) + 'static + Clone,
    {
        self.component.view().into_gpui(handle_msg)
    }
}

impl<C: Component> std::ops::Deref for GpuiMessageBridge<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<C: Component> std::ops::DerefMut for GpuiMessageBridge<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

/// Run an auto-ui Component with GPUI backend
///
/// This is the unified entry point for running auto-ui applications with GPUI.
/// It's called by `auto_ui::App::run()` when the "gpui" feature is enabled.
///
/// # Example
/// ```no_run
/// use auto_ui::{Component, View};
/// use auto_ui_gpui::run_app;
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     type Msg = ();
///     fn on(&mut self, _msg: Self::Msg) {}
///     fn view(&self) -> View<Self::Msg> {
///         View::text("Hello!")
///     }
/// }
///
/// fn main() -> auto_ui::AppResult<()> {
///     run_app::<MyComponent>()
/// }
/// ```
///
/// # Note
/// This function requires the Component to implement its own `Render` trait
/// implementation for GPUI. The auto-ui View abstraction is not used directly
/// due to GPUI's different architecture (closures vs message enums).
pub fn run_app<C>() -> auto_ui::AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + 'static,
{
    // Note: This is a placeholder. GPUI requires a different architecture
    // where the Component implements GPUI's Render trait directly.
    // See counter.rs example for the proper pattern.
    Err("GPUI backend requires manual implementation. See auto-ui-gpui-examples for patterns.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    enum TestMessage {
        Click,
        Toggle(bool),
    }

    #[test]
    fn test_text_conversion() {
        let view = AbstractView::text("Hello".to_string());
        let element = view.into_gpui_static();
        // Just ensure it compiles
    }

    #[test]
    fn test_button_conversion() {
        let view = AbstractView::button("Click me", TestMessage::Click);
        let element = view.into_gpui(|msg| {
            println!("Received message: {:?}", msg);
        });
        // Just ensure it compiles
    }

    #[test]
    fn test_column_conversion() {
        let view = AbstractView::col()
            .spacing(10)
            .padding(20)
            .child(AbstractView::text("Item 1"))
            .child(AbstractView::button("Click", TestMessage::Click))
            .build();

        let element = view.into_gpui(|msg| {
            println!("Received message: {:?}", msg);
        });
        // Just ensure it compiles
    }

    #[test]
    fn test_checkbox_conversion() {
        let view = AbstractView::checkbox(true, "Check me").on_toggle(TestMessage::Toggle);
        let element = view.into_gpui(|msg| {
            println!("Received message: {:?}", msg);
        });
        // Just ensure it compiles
    }
}
