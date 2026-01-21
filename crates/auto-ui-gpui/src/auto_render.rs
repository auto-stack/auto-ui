// Auto-conversion from enum-based messages to GPUI closures
//
// This module provides automatic conversion from auto-ui's enum-based
// message system to GPUI's closure-based event handling.

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, *};
use std::fmt::Debug;

/// Internal state holder for GPUI rendering
pub struct GpuiComponentState<C: Component> {
    pub component: C,
}

impl<C: Component> GpuiComponentState<C> {
    pub fn new(component: C) -> Self {
        Self { component }
    }

    /// Handle a message and update the component
    pub fn handle(&mut self, msg: C::Msg) {
        self.component.on(msg);
    }

    /// Get a reference to the component
    pub fn component(&self) -> &C {
        &self.component
    }

    /// Get a mutable reference to the component
    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }
}

// Implement Render trait for GpuiComponentState so it can be used directly in GPUI
impl<C: Component + 'static> Render for GpuiComponentState<C>
where
    C::Msg: Clone + Debug + 'static,
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get the view from the component and render it with auto message handling
        self.component.view().render_gpui_with(self, cx)
    }
}

/// Extended View trait with Context-aware rendering
pub trait ViewExt<M: Clone + Debug + 'static> {
    /// Convert View to GPUI element with automatic message handling
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static;
}

impl<M: Clone + Debug + 'static> ViewExt<M> for View<M> {
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static,
    {
        // Note: We can't create a simple closure here because:
        // 1. state.handle() requires &mut self
        // 2. cx.notify() requires &mut self
        // 3. Closures would need to be FnMut, but GPUI needs Fn
        //
        // Solution: Pass the needed context through the conversion
        self.clone().into_gpui_impl_with_context(state, cx)
    }
}

/// Internal trait for GPUI conversion with handler closure
pub trait IntoGpuiElementWithHandler<M: Clone + Debug + 'static> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static;

    fn into_gpui_impl_with_context<C>(
        self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static;
}

impl<M: Clone + Debug + 'static> IntoGpuiElementWithHandler<M> for View<M> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static,
    {
        // Use a counter for unique button IDs
        let mut button_counter = 0u64;

        // This is the simple version without context - used for static rendering
        match self {
            View::Empty => div().into_any(),

            View::Text(content) => div().child(content).into_any(),

            View::Button { label, onclick } => {
                let msg = onclick;
                let handle_msg_clone = handle_msg.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;
                Button::new(id)
                    .primary()
                    .label(label)
                    .on_click(move |_event, _window, _cx| {
                        handle_msg_clone(msg.clone());
                    })
                    .into_any_element()
            }

            View::Row { children, spacing, padding } => {
                let mut row_div = div().h_flex().gap(px(spacing as f32)).p(px(padding as f32));
                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding } => {
                let mut col_div = div().v_flex().gap(px(spacing as f32)).p(px(padding as f32));
                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, .. } => {
                div().child(format!("{}: {}", placeholder, value)).into_any()
            }

            View::Checkbox { is_checked, label, .. } => {
                div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label)).into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y } => {
                let handle_msg_clone = handle_msg.clone();
                let mut container_div = div().p(px(padding as f32));
                if let Some(w) = width {
                    container_div = container_div.w(px(w as f32));
                }
                if let Some(h) = height {
                    container_div = container_div.h(px(h as f32));
                }
                if center_x {
                    container_div = container_div.items_center();
                }
                if center_y {
                    container_div = container_div.justify_center();
                }
                container_div.child(child.clone().into_gpui_impl(handle_msg_clone)).into_any()
            }

            View::Scrollable { child, width, height } => {
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.clone().into_gpui_impl(handle_msg_clone);
                let mut scroll_div = div();
                if let Some(w) = width {
                    scroll_div = scroll_div.w(px(w as f32));
                }
                if let Some(h) = height {
                    scroll_div = scroll_div.h(px(h as f32));
                }
                scroll_div.child(child_element).into_any()
            }

            View::Radio { label, is_selected, .. } => {
                div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label)).into_any()
            }

            View::Select { options, selected_index, .. } => {
                let selected = selected_index.and_then(|i| options.get(i).cloned()).unwrap_or_default();
                div().child(format!("Select: {}", selected)).into_any()
            }

            View::List { items, spacing } => {
                let mut list_div = div().v_flex().gap(px(spacing as f32));
                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl(handle_msg.clone()));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing } => {
                let mut table_div = div().v_flex().gap(px(spacing as f32));

                let mut header_row_div = div().h_flex().gap(px(col_spacing as f32));
                for header in headers {
                    header_row_div = header_row_div.child(header.clone().into_gpui_impl(handle_msg.clone()));
                }
                table_div = table_div.child(header_row_div);

                for row_data in rows {
                    let mut row_div = div().h_flex().gap(px(col_spacing as f32));
                    for cell in row_data {
                        row_div = row_div.child(cell.clone().into_gpui_impl(handle_msg.clone()));
                    }
                    table_div = table_div.child(row_div);
                }

                table_div.into_any()
            }
        }
    }

    fn into_gpui_impl_with_context<C>(
        self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static,
    {
        // Use a counter for unique button IDs
        let mut button_counter = 0u64;

        // This version has access to state and cx for proper message handling
        match self {
            View::Empty => div().into_any(),

            View::Text(content) => div().child(content).into_any(),

            View::Button { label, onclick } => {
                let msg = onclick.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;
                Button::new(id)
                    .primary()
                    .label(label)
                    .on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                        state.handle(msg.clone());
                        _cx.notify();
                    }))
                    .into_any_element()
            }

            View::Row { children, spacing, padding } => {
                let mut row_div = div().h_flex().gap(px(spacing as f32)).p(px(padding as f32));
                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding } => {
                let mut col_div = div().v_flex().gap(px(spacing as f32)).p(px(padding as f32));
                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, .. } => {
                div().child(format!("{}: {}", placeholder, value)).into_any()
            }

            View::Checkbox { is_checked, label, .. } => {
                div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label)).into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y } => {
                let mut container_div = div().p(px(padding as f32));
                if let Some(w) = width {
                    container_div = container_div.w(px(w as f32));
                }
                if let Some(h) = height {
                    container_div = container_div.h(px(h as f32));
                }
                if center_x {
                    container_div = container_div.items_center();
                }
                if center_y {
                    container_div = container_div.justify_center();
                }
                container_div.child(child.clone().into_gpui_impl_with_context(state, cx)).into_any()
            }

            View::Scrollable { child, width, height } => {
                let child_element = child.clone().into_gpui_impl_with_context(state, cx);
                let mut scroll_div = div();
                if let Some(w) = width {
                    scroll_div = scroll_div.w(px(w as f32));
                }
                if let Some(h) = height {
                    scroll_div = scroll_div.h(px(h as f32));
                }
                scroll_div.child(child_element).into_any()
            }

            View::Radio { label, is_selected, .. } => {
                div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label)).into_any()
            }

            View::Select { options, selected_index, .. } => {
                let selected = selected_index.and_then(|i| options.get(i).cloned()).unwrap_or_default();
                div().child(format!("Select: {}", selected)).into_any()
            }

            View::List { items, spacing } => {
                let mut list_div = div().v_flex().gap(px(spacing as f32));
                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl_with_context(state, cx));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing } => {
                let mut table_div = div().v_flex().gap(px(spacing as f32));

                let mut header_row_div = div().h_flex().gap(px(col_spacing as f32));
                for header in headers {
                    header_row_div = header_row_div.child(header.clone().into_gpui_impl_with_context(state, cx));
                }
                table_div = table_div.child(header_row_div);

                for row_data in rows {
                    let mut row_div = div().h_flex().gap(px(col_spacing as f32));
                    for cell in row_data {
                        row_div = row_div.child(cell.clone().into_gpui_impl_with_context(state, cx));
                    }
                    table_div = table_div.child(row_div);
                }

                table_div.into_any()
            }
        }
    }
}
