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

            AbstractView::Text(content) => {
                // Direct text rendering
                text(content).into()
            }

            AbstractView::Button { label, onclick } => {
                // Button with click handler
                button(text(label))
                    .on_press(onclick)
                    .into()
            }

            AbstractView::Row { children, spacing, padding } => {
                let mut row_widget = row([]);
                row_widget = row_widget.spacing(spacing as f32);
                row_widget = row_widget.padding(padding as f32);

                // Recursively convert children
                for child in children {
                    row_widget = row_widget.push(child.into_iced());
                }

                row_widget.into()
            }

            AbstractView::Column { children, spacing, padding } => {
                let mut col_widget = column([]);
                col_widget = col_widget.spacing(spacing as f32);
                col_widget = col_widget.padding(padding as f32);

                // Recursively convert children
                for child in children {
                    col_widget = col_widget.push(child.into_iced());
                }

                col_widget.into()
            }

            AbstractView::Input { placeholder, value, on_change } => {
                // We need owned data for the text input, so we'll use placeholder as hint
                // and value as the current content
                let input_widget = text_input(&placeholder, &value);

                // Add change handler if provided
                if let Some(msg) = on_change {
                    input_widget.on_input(move |_| msg.clone()).into()
                } else {
                    input_widget.into()
                }
            }

            AbstractView::Checkbox { is_checked, label, on_toggle } => {
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

            AbstractView::Scrollable { child, width, height } => {
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
            } => {
                // Iced's pick_list for dropdown selection
                let selected_value = selected_index.and_then(|i| options.get(i).cloned());

                // We need to handle the case where on_select is None
                // Since Iced's pick_list requires a closure that returns a message,
                // we need to ensure on_select is always Some for functional selects
                match on_select {
                    Some(msg) => {
                        let picklist_widget = pick_list(options, selected_value, move |_| {
                            msg.clone()
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

            AbstractView::List { items, spacing } => {
                // List is essentially a column with spacing
                let mut col_widget = column([]);
                col_widget = col_widget.spacing(spacing as f32);

                // Recursively convert items
                for item in items {
                    col_widget = col_widget.push(item.into_iced());
                }

                col_widget.into()
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
        let element = view.into_iced();
        // Just ensure it compiles
    }

    #[test]
    fn test_button_conversion() {
        let view = AbstractView::button("Click me", TestMessage::Click);
        let element = view.into_iced();
    }

    #[test]
    fn test_column_conversion() {
        let view = AbstractView::col()
            .spacing(10)
            .padding(20)
            .child(AbstractView::text("Item 1"))
            .child(AbstractView::button("Click", TestMessage::Click))
            .build();

        let element = view.into_iced();
    }

    #[test]
    fn test_checkbox_conversion() {
        let view = AbstractView::checkbox(true, "Check me")
            .on_toggle(TestMessage::Toggle);
        let element = view.into_iced();
    }
}
