// auto-ui-gpui: GPUI backend adapter for auto-ui
//
// This crate provides adapter traits to convert auto-ui's abstract View<M>
// into GPUI's render tree using gpui-component library.
//
// Phase 2 Integration: Now supports unified styling system with Style objects.

#![recursion_limit = "512"]

use auto_ui::{View as AbstractView, Component, Style};
use auto_ui::style::gpui_adapter::GpuiStyle;
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, scroll::ScrollableElement, *};
use std::fmt::Debug;

// Auto-conversion module
pub mod auto_render;
pub use auto_render::{GpuiComponentState, ViewExt};

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

            AbstractView::Text { content, style } => {
                // Direct text rendering with optional styling
                let mut text_div = div().child(content);
                if let Some(style) = style {
                    text_div = apply_gpui_style_to_div(text_div, &style);
                }
                text_div.into_any()
            }

            AbstractView::Button { label, onclick: _, style } => {
                // Button with click handler - note: we can't directly handle messages
                // in GPUI's Button without proper context. This is a simplified version.
                button_counter += 1;
                let label_clone = label.clone();
                let mut button = Button::new(("button", button_counter))
                    .label(label_clone);

                // Apply style if present
                if style.is_some() {
                    // Note: Simplified styling for this version
                    button = button.primary();
                } else {
                    button = button.primary();
                }

                button.into_any_element()
            }

            AbstractView::Row { children, spacing, padding, style } => {
                let mut row_div = div().h_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    row_div = apply_gpui_style_to_div(row_div, &style);
                } else {
                    // Legacy API support
                    row_div = row_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                // Recursively convert children
                for child in children {
                    let handle_msg_clone = handle_msg.clone();
                    row_div = row_div.child(child.into_gpui(handle_msg_clone));
                }

                row_div.into_any()
            }

            AbstractView::Column { children, spacing, padding, style } => {
                let mut col_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    col_div = apply_gpui_style_to_div(col_div, &style);
                } else {
                    // Legacy API support
                    col_div = col_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

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
                style,
            } => {
                // GPUI text input
                // Note: gpui-component's Input implementation requires more complex setup
                // For now, we use a simple div that shows the value
                let mut input_div = div().child(format!("{}: {}", placeholder, value));
                if let Some(style) = style {
                    input_div = apply_gpui_style_to_div(input_div, &style);
                }
                input_div.into_any()
            }

            AbstractView::Checkbox {
                is_checked,
                label,
                on_toggle: _,
                style,
            } => {
                // Checkbox with label - simplified version
                let display_text = format!(
                    "{} {}",
                    if is_checked { "[✓]" } else { "[ ]" },
                    label
                );

                let mut checkbox_div = div().child(display_text);
                if let Some(style) = style {
                    checkbox_div = apply_gpui_style_to_div(checkbox_div, &style);
                }
                checkbox_div.into_any()
            }

            AbstractView::Container {
                child,
                padding,
                width,
                height,
                center_x,
                center_y,
                style,
            } => {
                let mut container_div = div();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    container_div = apply_gpui_style_to_div(container_div, &style);
                } else {
                    // Legacy API support
                    container_div = container_div.p(px(padding as f32));

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
                style,
            } => {
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.into_gpui(handle_msg_clone);

                // Apply styling before wrapping in scrollable
                let mut inner_div = div();
                if let Some(style) = style {
                    inner_div = apply_gpui_style_to_div(inner_div, &style);
                } else {
                    // Legacy API support
                    if let Some(w) = width {
                        inner_div = inner_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        inner_div = inner_div.h(px(h as f32));
                    }
                }

                let mut scroll_div = inner_div.child(child_element).overflow_scrollbar();
                scroll_div.into_any_element()
            }

            AbstractView::Radio {
                label,
                is_selected,
                on_select: _,
                style,
            } => {
                // Radio button - simplified version
                let display_text = format!(
                    "{} {}",
                    if is_selected { "(•)" } else { "( )" },
                    label
                );

                let mut radio_div = div().child(display_text);
                if let Some(style) = style {
                    radio_div = apply_gpui_style_to_div(radio_div, &style);
                }
                radio_div.into_any()
            }

            AbstractView::Select {
                options,
                selected_index,
                on_select: _,
                style,
            } => {
                // Select dropdown - simplified version
                let selected = selected_index
                    .and_then(|i| options.get(i).cloned())
                    .unwrap_or_default();

                let mut select_div = div().child(format!("Select: {}", selected));
                if let Some(style) = style {
                    select_div = apply_gpui_style_to_div(select_div, &style);
                }
                select_div.into_any()
            }

            AbstractView::List { items, spacing, style } => {
                let mut list_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    list_div = apply_gpui_style_to_div(list_div, &style);
                } else {
                    // Legacy API support
                    list_div = list_div.gap(px(spacing as f32));
                }

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
                style,
            } => {
                let mut table_div = div().v_flex();

                // Apply unified styling if present
                if let Some(style) = style {
                    table_div = apply_gpui_style_to_div(table_div, &style);
                } else {
                    // Legacy API support
                    table_div = table_div.gap(px(spacing as f32));
                }

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

/// Apply a Style to a GPUI div element (simplified version for lib.rs)
fn apply_gpui_style_to_div(div: Div, style: &Style) -> Div {
    let gpui_style = GpuiStyle::from_style(style);
    let mut result = div;

    // Apply spacing (L1 + L2)
    if let Some(gpui_padding) = &gpui_style.padding {
        match gpui_padding {
            auto_ui::style::gpui_adapter::GpuiPadding::Uniform(size) => {
                result = result.p(px(*size));
            }
        }
    }
    if let Some(padding_x) = gpui_style.padding_x {
        result = result.px(px(padding_x));
    }
    if let Some(padding_y) = gpui_style.padding_y {
        result = result.py(px(padding_y));
    }
    if let Some(gap) = gpui_style.gap {
        result = result.gap(px(gap));
    }

    // Apply colors (L1)
    if let Some(bg_color) = gpui_style.background_color {
        result = result.bg(bg_color);
    }
    if let Some(text_color) = gpui_style.text_color {
        result = result.text_color(text_color);
    }

    // Apply border radius (L1 + L2)
    if gpui_style.rounded {
        if let Some(rounded_size) = gpui_style.rounded_size {
            match rounded_size {
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Sm => {
                    result = result.rounded(px(2.0));
                }
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Md => {
                    result = result.rounded(px(4.0));
                }
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Lg => {
                    result = result.rounded(px(8.0));
                }
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Xl => {
                    result = result.rounded(px(12.0));
                }
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Xxl => {
                    result = result.rounded(px(16.0));
                }
                auto_ui::style::gpui_adapter::GpuiRoundedSize::Full => {
                    result = result.rounded(px(9999.0));
                }
            }
        }
    }

    result
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
/// This function provides automatic conversion from enum-based messages
/// to GPUI's closure-based event handling.
///
/// # Example
/// ```no_run
/// use auto_ui::{Component, View};
/// use auto_ui_gpui::run_app;
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     type Msg = MyMessage;
///     fn on(&mut self, msg: Self::Msg) { /* ... */ }
///     fn view(&self) -> View<Self::Msg> { /* ... */ }
/// }
///
/// fn main() -> auto_ui::AppResult<()> {
///     run_app::<MyComponent>("My App")
/// }
/// ```
pub fn run_app<C>(title: &str) -> auto_ui::AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + 'static,
{
    use auto_render::GpuiComponentState;

    // Convert title to owned String to avoid lifetime issues
    let title = title.to_owned();

    // Run GPUI application
    let app = gpui::Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(100.0), y: px(100.0) },
                        size: gpui::Size {
                            width: px(800.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title.into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    // Create the state as a GPUI entity
                    let state = cx.new(|_| GpuiComponentState::new(C::default()));

                    // Build the UI using the state's render implementation
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(state, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
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
