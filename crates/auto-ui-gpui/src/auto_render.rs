// Auto-conversion from enum-based messages to GPUI closures
//
// This module provides automatic conversion from auto-ui's enum-based
// message system to GPUI's closure-based event handling.
//
// Phase 2 Integration: Now supports unified styling system with Style objects.

use auto_ui::{Component, View, Style};
use auto_ui::style::gpui_adapter::{GpuiStyle, GpuiFontSize, GpuiFontWeight, GpuiTextAlign};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, scroll::ScrollableElement, *};
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

            View::Text { content, style } => {
                let mut text_div = div().child(content);
                // Apply unified styling if present
                if let Some(style) = style {
                    text_div = apply_style_to_div(text_div, &style);
                }
                text_div.into_any()
            }

            View::Button { label, onclick, style } => {
                let msg = onclick;
                let handle_msg_clone = handle_msg.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;

                // Apply unified styling if present
                let mut button = Button::new(id).label(label);
                if let Some(style) = style {
                    button = apply_style_to_button(button, &style);
                } else {
                    button = button.primary(); // Default style
                }

                button.on_click(move |_event, _window, _cx| {
                    handle_msg_clone(msg.clone());
                })
                .into_any_element()
            }

            View::Row { children, spacing, padding, style } => {
                let mut row_div = div().h_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    row_div = apply_style_to_div(row_div, &style);
                } else {
                    // Legacy API support
                    row_div = row_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding, style } => {
                let mut col_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    col_div = apply_style_to_div(col_div, &style);
                } else {
                    // Legacy API support
                    col_div = col_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, style, .. } => {
                let mut input_div = div().child(format!("{}: {}", placeholder, value));
                // Apply unified styling if present
                if let Some(style) = style {
                    input_div = apply_style_to_div(input_div, &style);
                }
                input_div.into_any()
            }

            View::Checkbox { is_checked, label, style, .. } => {
                let mut checkbox_div = div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    checkbox_div = apply_style_to_div(checkbox_div, &style);
                }
                checkbox_div.into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y, style } => {
                let handle_msg_clone = handle_msg.clone();
                let mut container_div = div();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    container_div = apply_style_to_div(container_div, &style);
                } else {
                    // Legacy API support
                    container_div = container_div.p(px(padding as f32));
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
                }

                container_div.child(child.clone().into_gpui_impl(handle_msg_clone)).into_any()
            }

            View::Scrollable { child, width, height, style } => {
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.clone().into_gpui_impl(handle_msg_clone);

                // Apply styling before wrapping in scrollable
                let mut inner_div = div();
                if let Some(style) = style {
                    inner_div = apply_style_to_div(inner_div, &style);
                } else {
                    // Legacy API support
                    if let Some(w) = width {
                        inner_div = inner_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        inner_div = inner_div.h(px(h as f32));
                    }
                }

                inner_div.child(child_element).overflow_scrollbar().into_any_element()
            }

            View::Radio { label, is_selected, style, .. } => {
                let mut radio_div = div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    radio_div = apply_style_to_div(radio_div, &style);
                }
                radio_div.into_any()
            }

            View::Select { options, selected_index, style, .. } => {
                let selected = selected_index.and_then(|i| options.get(i).cloned()).unwrap_or_default();
                let mut select_div = div().child(format!("Select: {}", selected));
                // Apply unified styling if present
                if let Some(style) = style {
                    select_div = apply_style_to_div(select_div, &style);
                }
                select_div.into_any()
            }

            View::List { items, spacing, style } => {
                let mut list_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    list_div = apply_style_to_div(list_div, &style);
                } else {
                    // Legacy API support
                    list_div = list_div.gap(px(spacing as f32));
                }

                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl(handle_msg.clone()));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing, style } => {
                let mut table_div = div().v_flex();

                // Apply unified styling if present
                if let Some(style) = style {
                    table_div = apply_style_to_div(table_div, &style);
                } else {
                    // Legacy API support
                    table_div = table_div.gap(px(spacing as f32));
                }

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

            View::Text { content, style } => {
                let mut text_div = div().child(content);
                // Apply unified styling if present
                if let Some(style) = style {
                    text_div = apply_style_to_div(text_div, &style);
                }
                text_div.into_any()
            }

            View::Button { label, onclick, style } => {
                let msg = onclick.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;

                // Apply unified styling if present
                let mut button = Button::new(id).label(label);
                if let Some(style) = style {
                    button = apply_style_to_button(button, &style);
                } else {
                    button = button.primary(); // Default style
                }

                button.on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                    state.handle(msg.clone());
                    _cx.notify();
                }))
                .into_any_element()
            }

            View::Row { children, spacing, padding, style } => {
                let mut row_div = div().h_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    row_div = apply_style_to_div(row_div, &style);
                } else {
                    // Legacy API support
                    row_div = row_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding, style } => {
                let mut col_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    col_div = apply_style_to_div(col_div, &style);
                } else {
                    // Legacy API support
                    col_div = col_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, style, .. } => {
                let mut input_div = div().child(format!("{}: {}", placeholder, value));
                // Apply unified styling if present
                if let Some(style) = style {
                    input_div = apply_style_to_div(input_div, &style);
                }
                input_div.into_any()
            }

            View::Checkbox { is_checked, label, style, .. } => {
                let mut checkbox_div = div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    checkbox_div = apply_style_to_div(checkbox_div, &style);
                }
                checkbox_div.into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y, style } => {
                let mut container_div = div();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    container_div = apply_style_to_div(container_div, &style);
                } else {
                    // Legacy API support
                    container_div = container_div.p(px(padding as f32));
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
                }

                container_div.child(child.clone().into_gpui_impl_with_context(state, cx)).into_any()
            }

            View::Scrollable { child, width, height, style } => {
                let child_element = child.clone().into_gpui_impl_with_context(state, cx);

                // Apply styling before wrapping in scrollable
                let mut inner_div = div();
                if let Some(style) = style {
                    inner_div = apply_style_to_div(inner_div, &style);
                } else {
                    // Legacy API support
                    if let Some(w) = width {
                        inner_div = inner_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        inner_div = inner_div.h(px(h as f32));
                    }
                }

                inner_div.child(child_element).overflow_scrollbar().into_any_element()
            }

            View::Radio { label, is_selected, style, .. } => {
                let mut radio_div = div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    radio_div = apply_style_to_div(radio_div, &style);
                }
                radio_div.into_any()
            }

            View::Select { options, selected_index, style, .. } => {
                let selected = selected_index.and_then(|i| options.get(i).cloned()).unwrap_or_default();
                let mut select_div = div().child(format!("Select: {}", selected));
                // Apply unified styling if present
                if let Some(style) = style {
                    select_div = apply_style_to_div(select_div, &style);
                }
                select_div.into_any()
            }

            View::List { items, spacing, style } => {
                let mut list_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    list_div = apply_style_to_div(list_div, &style);
                } else {
                    // Legacy API support
                    list_div = list_div.gap(px(spacing as f32));
                }

                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl_with_context(state, cx));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing, style } => {
                let mut table_div = div().v_flex();

                // Apply unified styling if present
                if let Some(style) = style {
                    table_div = apply_style_to_div(table_div, &style);
                } else {
                    // Legacy API support
                    table_div = table_div.gap(px(spacing as f32));
                }

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

/// Apply a Style to a GPUI div element
fn apply_style_to_div(div: Div, style: &Style) -> Div {
    let gpui_style = GpuiStyle::from_style(style);
    let mut result = div;

    // Apply spacing (L1 + L2)
    if let Some(padding) = &gpui_style.padding {
        match padding {
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
    if let Some(margin) = gpui_style.margin {
        result = result.m(px(margin));
    }
    if let Some(margin_x) = gpui_style.margin_x {
        result = result.mx(px(margin_x));
    }
    if let Some(margin_y) = gpui_style.margin_y {
        result = result.my(px(margin_y));
    }
    if let Some(gap) = gpui_style.gap {
        result = result.gap(px(gap));
    }

    // Apply layout (L1 + L2)
    if let Some(flex) = gpui_style.flex {
        if flex {
            // Already set by h_flex() or v_flex()
        }
    }
    if let Some(flex_direction) = gpui_style.flex_direction {
        match flex_direction {
            auto_ui::style::gpui_adapter::GpuiFlexDirection::Row => {
                result = result.h_flex();
            }
            auto_ui::style::gpui_adapter::GpuiFlexDirection::Col => {
                result = result.v_flex();
            }
        }
    }
    if let Some(items_align) = gpui_style.items_align {
        match items_align {
            auto_ui::style::gpui_adapter::GpuiAlignment::Center => {
                result = result.items_center();
            }
            auto_ui::style::gpui_adapter::GpuiAlignment::Start => {
                result = result.items_start();
            }
            auto_ui::style::gpui_adapter::GpuiAlignment::End => {
                result = result.items_end();
            }
            _ => {}
        }
    }
    if let Some(justify_align) = gpui_style.justify_align {
        match justify_align {
            auto_ui::style::gpui_adapter::GpuiAlignment::Center => {
                result = result.justify_center();
            }
            auto_ui::style::gpui_adapter::GpuiAlignment::Between => {
                result = result.justify_between();
            }
            auto_ui::style::gpui_adapter::GpuiAlignment::Start => {
                result = result.justify_start();
            }
            auto_ui::style::gpui_adapter::GpuiAlignment::End => {
                result = result.justify_end();
            }
        }
    }

    // Apply sizing (L1)
    if let Some(width) = gpui_style.width {
        match width {
            auto_ui::style::gpui_adapter::GpuiSize::Full => {
                result = result.w(px(300.0)); // Arbitrary full size
            }
            auto_ui::style::gpui_adapter::GpuiSize::Fixed(size) => {
                result = result.w(px(size));
            }
        }
    }
    if let Some(height) = gpui_style.height {
        match height {
            auto_ui::style::gpui_adapter::GpuiSize::Full => {
                result = result.h(px(300.0)); // Arbitrary full size
            }
            auto_ui::style::gpui_adapter::GpuiSize::Fixed(size) => {
                result = result.h(px(size));
            }
        }
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

    // Apply border (L2)
    if gpui_style.border {
        if let Some(border_width) = gpui_style.border_width {
            if let Some(border_color) = gpui_style.border_color {
                result = result.border(px(border_width)).border_color(border_color);
            }
        }
    }

    // Apply shadow (L3)
    if gpui_style.shadow {
        if let Some(shadow_size) = gpui_style.shadow_size {
            match shadow_size {
                auto_ui::style::gpui_adapter::GpuiShadowSize::Sm => {
                    result = result.shadow_lg();
                }
                auto_ui::style::gpui_adapter::GpuiShadowSize::Md => {
                    result = result.shadow_lg();
                }
                auto_ui::style::gpui_adapter::GpuiShadowSize::Lg => {
                    result = result.shadow_xl();
                }
                _ => {
                    result = result.shadow_lg();
                }
            }
        }
    }

    // Apply opacity (L3)
    if let Some(opacity) = gpui_style.opacity {
        result = result.opacity(opacity);
    }

    result
}

/// Apply a Style to a GPUI Button element
fn apply_style_to_button(button: Button, style: &Style) -> Button {
    let gpui_style = GpuiStyle::from_style(style);
    let mut result = button;

    // Apply colors (L1)
    if let Some(_bg_color) = gpui_style.background_color {
        // Note: GPUI Button component may not support custom background colors in this version
        // Using primary variant as fallback
        result = result.primary();
    }

    // Apply text color (L1)
    if let Some(_text_color) = gpui_style.text_color {
        // Note: GPUI Button component may not support custom text colors in this version
    }

    // Apply border radius (L1 + L2) - use size variants as approximation
    if gpui_style.rounded {
        if let Some(_rounded_size) = gpui_style.rounded_size {
            // GPUI-Component Button doesn't expose rounded size directly
            // Use primary as a styled variant
            result = result.primary();
        }
    }

    // Apply typography (L2) - Button variants
    if let Some(font_weight) = gpui_style.font_weight {
        match font_weight {
            GpuiFontWeight::Bold => {
                result = result.primary(); // Bold variant
            }
            GpuiFontWeight::Medium => {
                result = result.primary(); // No medium variant, use primary
            }
            GpuiFontWeight::Normal => {
                result = result.ghost(); // Normal variant
            }
        }
    } else if gpui_style.background_color.is_some() {
        // If there's a background color but no font weight, use primary
        result = result.primary();
    }

    // Apply shadow (L3)
    if gpui_style.shadow {
        // Note: GPUI Button may not support custom shadows
    }

    result
}
