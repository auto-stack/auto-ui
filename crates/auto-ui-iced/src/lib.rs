// auto-ui-iced: Iced backend adapter for auto-ui
//
// This crate provides adapter traits to convert auto-ui's abstract View<M>
// into iced's Element for rendering.

use auto_ui::{View as AbstractView, Component};
use std::fmt::Debug;
use iced::widget::{button, checkbox, column, pick_list, row, text, text_input};

/// Trait for converting abstract View<M> into iced Element
///
/// This trait enables rendering auto-ui's abstract view tree using iced framework.
///
/// # Example
/// ```rust
/// use auto_ui::View;
/// use auto_ui_iced::IntoIcedElement;
///
/// let view: View<MyMessage> = View::button("Click", MyMessage::Click);
/// let iced_element = view.into_iced();
/// ```
pub trait IntoIcedElement<M: Clone + Debug + 'static> {
    /// Convert abstract view into iced Element
    fn into_iced(self) -> iced::Element<'static, M>;
}

impl<M: Clone + Debug + 'static> IntoIcedElement<M> for AbstractView<M> {
    fn into_iced(self) -> iced::Element<'static, M> {
        match self {
            AbstractView::Empty => {
                // Empty view renders as empty text
                text("").into()
            }

            AbstractView::Text { content, style: _ } => {
                // Direct text rendering (style ignored for now in Iced)
                text(content).into()
            }

            AbstractView::Button { label, onclick, style: _ } => {
                // Button with click handler (style ignored for now)
                button(text(label))
                    .on_press(onclick)
                    .into()
            }

            AbstractView::Row { children, spacing, padding, style: _ } => {
                let mut row_widget = row([]);
                row_widget = row_widget.spacing(spacing as f32);
                row_widget = row_widget.padding(padding as f32);

                // Recursively convert children
                for child in children {
                    row_widget = row_widget.push(child.into_iced());
                }

                row_widget.into()
            }

            AbstractView::Column { children, spacing, padding, style: _ } => {
                let mut col_widget = column([]);
                col_widget = col_widget.spacing(spacing as f32);
                col_widget = col_widget.padding(padding as f32);

                // Recursively convert children
                for child in children {
                    col_widget = col_widget.push(child.into_iced());
                }

                col_widget.into()
            }

            AbstractView::Input {
                placeholder,
                value,
                on_change,
                width,
                password: _,
                style: _,
            } => {
                // Create text input widget
                let mut input_widget = text_input(&placeholder, &value);

                // Apply width
                if let Some(w) = width {
                    input_widget = input_widget.width(iced::Length::Fixed(w as f32));
                }

                // Add change handler if provided
                if let Some(msg) = on_change {
                    input_widget.on_input(move |_| msg.clone()).into()
                } else {
                    input_widget.into()
                }
            }

            AbstractView::Checkbox { is_checked, label, on_toggle, style: _ } => {
                // Checkbox with label - use row to combine checkbox and text
                let checkbox_widget = checkbox(is_checked);

                // Add toggle handler if provided
                let checkbox_with_handler = if let Some(msg) = on_toggle {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Combine with label in a row
                row![checkbox_with_handler, text(label)]
                    .spacing(4)
                    .into()
            }

            AbstractView::Container {
                child,
                padding,
                width,
                height,
                center_x,
                center_y,
                style: _,
            } => {
                use iced::widget::container;

                let mut container_widget = container(child.into_iced());

                // Apply padding
                if padding > 0 {
                    container_widget = container_widget.padding(padding as f32);
                }

                // Apply width
                if let Some(w) = width {
                    container_widget = container_widget.width(iced::Length::Fixed(w as f32));
                }

                // Apply height
                if let Some(h) = height {
                    container_widget = container_widget.height(iced::Length::Fixed(h as f32));
                }

                // Apply centering (aligns the container content)
                if center_x {
                    container_widget = container_widget.center_x(iced::Length::Fill);
                }
                if center_y {
                    container_widget = container_widget.center_y(iced::Length::Fill);
                }

                container_widget.into()
            }

            AbstractView::Scrollable { child, width, height, style: _ } => {
                use iced::widget::scrollable;

                let mut scrollable_widget = scrollable(child.into_iced());

                // Apply width
                if let Some(w) = width {
                    scrollable_widget = scrollable_widget.width(iced::Length::Fixed(w as f32));
                }

                // Apply height
                if let Some(h) = height {
                    scrollable_widget = scrollable_widget.height(iced::Length::Fixed(h as f32));
                }

                scrollable_widget.into()
            }

            AbstractView::Radio {
                label,
                is_selected,
                on_select,
                style: _,
            } => {
                // Note: Iced's radio widget requires a closure, which doesn't fit our
                // message-based abstraction. We simulate radio with checkbox styling.
                // In a real implementation, we'd use Iced's radio widget with proper closures.
                let checkbox_widget = checkbox(is_selected);

                // Add select handler if provided
                let checkbox_with_handler = if let Some(msg) = on_select {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Combine with label in a row
                row![checkbox_with_handler, text(label)]
                    .spacing(4)
                    .into()
            }

            AbstractView::Select {
                options,
                selected_index,
                on_select,
                style: _,
            } => {
                // Iced's pick_list for dropdown selection
                let selected_value = selected_index.and_then(|i| options.get(i).cloned());

                // Use the callback to handle selection
                match on_select {
                    Some(callback) => {
                        let options_clone = options.clone();
                        let picklist_widget = pick_list(options, selected_value, move |selected_string| {
                            // Find the index of the selected string
                            let index = options_clone.iter()
                                .position(|s| *s == selected_string)
                                .unwrap_or(0);
                            callback.call(index, selected_string.as_str())
                        });
                        picklist_widget.into()
                    }
                    None => {
                        // No handler - display selected value or first option as text
                        let display_text = selected_value
                            .unwrap_or_else(|| options.first().cloned().unwrap_or_default());
                        text(display_text).into()
                    }
                }
            }

            AbstractView::List { items, spacing, style: _ } => {
                // List is essentially a column with spacing
                let mut col_widget = column([]);
                col_widget = col_widget.spacing(spacing as f32);

                // Recursively convert items
                for item in items {
                    col_widget = col_widget.push(item.into_iced());
                }

                col_widget.into()
            }

            AbstractView::Table {
                headers,
                rows,
                spacing,
                col_spacing,
                style: _,
            } => {
                // Table is implemented as a column of rows
                let mut table_widget = column([]);
                table_widget = table_widget.spacing(spacing as f32);

                // Add header row
                let mut header_row_widget = row([]);
                header_row_widget = header_row_widget.spacing(col_spacing as f32);
                for header in headers {
                    header_row_widget = header_row_widget.push(header.into_iced());
                }
                table_widget = table_widget.push(header_row_widget);

                // Add data rows
                for row_data in rows {
                    let mut row_widget = row([]);
                    row_widget = row_widget.spacing(col_spacing as f32);
                    for cell in row_data {
                        row_widget = row_widget.push(cell.into_iced());
                    }
                    table_widget = table_widget.push(row_widget);
                }

                table_widget.into()
            }

            AbstractView::Slider {
                min,
                max,
                value,
                on_change,
                step,
                style: _,
            } => {
                use iced::widget::slider;
                // Create slider widget with proper value handling
                let mut slider_widget = slider(min..=max, value, on_change);

                // Apply step if specified
                if let Some(step_value) = step {
                    slider_widget = slider_widget.step(step_value);
                }

                slider_widget.into()
            }

            AbstractView::ProgressBar { progress, style: _ } => {
                use iced::widget::progress_bar;
                // Progress bar in Iced
                progress_bar(0.0..=1.0, progress).into()
            }

            // Plan 010: Unified Navigation Components - Iced Implementation

            AbstractView::Accordion {
                items,
                allow_multiple: _,
                on_toggle,
                style: _,
            } => {
                use auto_ui::AccordionItem;
                use iced::widget::container;

                // Accordion is implemented as a column of collapsible sections
                let mut accordion_widget = column([]);

                for (idx, item) in items.into_iter().enumerate() {
                    // Create header button with icon + title
                    let header_text = if let Some(icon) = item.icon {
                        format!("{} {}", icon, item.title)
                    } else {
                        item.title.clone()
                    };

                    let header_button = if let Some(callback) = &on_toggle {
                        // Toggle message - need to clone for closure
                        let callback_clone = callback.clone();
                        button(text(header_text))
                            .on_press(callback_clone.call(idx, !item.expanded))
                    } else {
                        button(text(header_text))
                    };

                    // Create children (if expanded)
                    let children_view: iced::Element<M> = if item.expanded && !item.children.is_empty() {
                        let mut children_col = column([]);
                        for child in item.children {
                            children_col = children_col.push(child.into_iced());
                        }
                        children_col.into()
                    } else {
                        text("").into()
                    };

                    // Combine header and children in a container
                    let section = container(column![header_button, children_view].spacing(4));
                    accordion_widget = accordion_widget.push(section);
                }

                container(accordion_widget).padding(10).into()
            }

            AbstractView::Sidebar {
                content,
                width,
                collapsible: _,
                position,
                style: _,
            } => {
                use iced::widget::container;
                use iced::Length;

                // Sidebar is a fixed-width container
                let mut sidebar_container = container(content.into_iced())
                    .width(Length::Fixed(width))
                    .height(Length::Fill);

                // Add border based on position
                sidebar_container = match position {
                    auto_ui::SidebarPosition::Left => sidebar_container,
                    auto_ui::SidebarPosition::Right => sidebar_container,
                };

                sidebar_container.into()
            }

            AbstractView::Tabs {
                labels,
                contents,
                selected,
                position: _,
                on_select: _,
                style: _,
            } => {
                use iced::widget::{container, column};
                use auto_ui::TabsPosition;

                // Tabs are implemented as column with tab buttons + selected content
                let mut tabs_widget = column([]);

                // Create tab buttons row
                let mut tab_buttons_row = row([]);
                for (idx, label) in labels.iter().enumerate() {
                    let is_selected = idx == selected;
                    let label_text = if is_selected {
                        format!("[{}]", label)  // Highlight selected
                    } else {
                        label.clone()
                    };

                    let tab_button = button(text(label_text));
                    tab_buttons_row = tab_buttons_row.push(tab_button);
                }

                tabs_widget = tabs_widget.push(tab_buttons_row);

                // Add selected content
                if let Some(content) = contents.get(selected) {
                    tabs_widget = tabs_widget.push(container(content.clone().into_iced()).padding(20));
                }

                container(tabs_widget).into()
            }

            AbstractView::NavigationRail {
                items,
                selected: _,
                width,
                show_labels,
                on_select: _,
                style: _,
            } => {
                use iced::widget::{container, column};
                use iced::Length;

                // NavigationRail is a compact vertical navigation
                let mut rail_widget = column([]);

                for item in items {
                    // Create navigation item with icon
                    let item_text = if show_labels {
                        format!("{}  {}", item.icon, item.label)
                    } else {
                        item.icon.to_string()
                    };

                    // Add badge if present
                    let item_text_with_badge = if let Some(badge) = &item.badge {
                        format!("{} ({})", item_text, badge)
                    } else {
                        item_text
                    };

                    let nav_button = button(text(item_text_with_badge));
                    rail_widget = rail_widget.push(nav_button);
                }

                container(rail_widget)
                    .width(Length::Fixed(width))
                    .height(Length::Fill)
                    .padding(10)
                    .into()
            }
        }
    }
}

/// Extension trait for Component to add iced-compatible view method
///
/// This allows components to be used directly with iced::run().
///
/// # Example
/// ```rust
/// use auto_ui::{Component, View};
/// use auto_ui_iced::ComponentIced;
///
/// struct MyComponent { ... }
///
/// impl Component for MyComponent {
///     type Msg = MyMessage;
///     fn on(&mut self, msg: Self::Msg) { ... }
///     fn view(&self) -> View<Self::Msg> { ... }
/// }
///
/// // Use with iced::run
/// iced::run("Title", MyComponent::update, MyComponent::view_iced)
/// ```
pub trait ComponentIced: Component {
    /// Iced-compatible view function
    fn view_iced(&self) -> iced::Element<'static, Self::Msg>;

    /// Iced-compatible update function (delegates to on())
    fn update(&mut self, msg: Self::Msg) {
        self.on(msg);
    }
}

// Blanket implementation for all Component types
impl<T: Component> ComponentIced for T
where
    T::Msg: Clone + Debug + 'static,
{
    fn view_iced(&self) -> iced::Element<'static, T::Msg> {
        self.view().into_iced()
    }
}

/// Run an auto-ui Component with Iced backend
///
/// This is the unified entry point for running auto-ui applications with Iced.
/// It's called by `auto_ui::App::run()` when the "iced" feature is enabled.
///
/// # Example
/// ```no_run
/// use auto_ui::{Component, View};
/// use auto_ui_iced::run_app;
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
pub fn run_app<C>() -> auto_ui::AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    Ok(iced::run(C::update, view)?)
}

fn view<C>(component: &C) -> iced::Element<'_, C::Msg>
where
    C: Component,
    C::Msg: Clone + Debug + 'static,
{
    component.view_iced()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    enum TestMessage {
        Click,
        Toggle(bool),
    }

    #[test]
    fn test_text_conversion() {
        let view = AbstractView::text("Hello".to_string());
        let _element = view.into_iced();
        // Just ensure it compiles
    }

    #[test]
    fn test_button_conversion() {
        let view = AbstractView::button("Click me", TestMessage::Click);
        let _element = view.into_iced();
    }

    #[test]
    fn test_column_conversion() {
        let view = AbstractView::col()
            .spacing(10)
            .padding(20)
            .child(AbstractView::text("Item 1"))
            .child(AbstractView::button("Click", TestMessage::Click))
            .build();

        let _element = view.into_iced();
    }

    #[test]
    fn test_checkbox_conversion() {
        let view = AbstractView::checkbox(true, "Check me")
            .on_toggle(TestMessage::Toggle);
        let _element = view.into_iced();
    }
}
