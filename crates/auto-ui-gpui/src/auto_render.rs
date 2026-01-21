// Auto-conversion from enum-based messages to GPUI closures
//
// This module provides automatic conversion from auto-ui's enum-based
// message system to GPUI's closure-based event handling.

use crate::{Component, View};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, *};
use std::fmt::Debug;
use std::rc::Rc;

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
        C: Component<Msg = M>;
}

impl<M: Clone + Debug + 'static> ViewExt<M> for View<M> {
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M>,
    {
        // Create a message handler that uses the Context
        let handle_msg = |msg: M| {
            state.handle(msg);
            cx.notify(); // Trigger re-render
        };

        self.clone().into_gpui_impl(handle_msg)
    }
}

/// Internal trait for GPUI conversion with handler closure
pub trait IntoGpuiElementWithHandler<M: Clone + Debug + 'static> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static;
}

impl<M: Clone + Debug + 'static> IntoGpuiElementWithHandler<M> for View<M> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static,
    {
        match self {
            View::Empty => div().into_any(),

            View::Text(content) => div().child(content).into_any(),

            View::Button { label, onclick } => {
                let msg = onclick.clone();
                Button::new(label.as_str())
                    .primary()
                    .label(label)
                    .on_click(move |_event, _window, _cx| {
                        handle_msg(msg.clone());
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
                div().child(format!("{} [{}]", if *is_checked { "✓" } else { " " }, label)).into_any()
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
                let mut scroll_div = div().overflow_scrollbar();
                if let Some(w) = width {
                    scroll_div = scroll_div.w(px(w as f32));
                }
                if let Some(h) = height {
                    scroll_div = scroll_div.h(px(h as f32));
                }
                scroll_div.child(child_element).into_any()
            }

            View::Radio { label, is_selected, .. } => {
                div().child(format!("{} ({})", if *is_selected { "•" } { " " }, label)).into_any()
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
}
