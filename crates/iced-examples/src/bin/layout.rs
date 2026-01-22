// Layout example for iced
// Demonstrates various layout patterns: rows, columns, spacing
use iced::widget::{center, column, row, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run(LayoutExample::update, LayoutExample::view)
}

#[derive(Default)]
struct LayoutExample;

#[derive(Debug, Clone, Copy)]
enum Message {
    NoOp,
}

impl LayoutExample {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        center(
            column![
                text("Layout Examples").size(32),
                text("Column Layout").size(24),
                column_layout_example(),
                text("Row Layout").size(24),
                row_layout_example(),
                text("Nested Layout").size(24),
                nested_layout_example(),
            ]
            .spacing(30)
            .padding(40),
        )
        .into()
    }
}

fn column_layout_example() -> Element<'static, Message> {
    column![
        text("Item 1"),
        text("Item 2"),
        text("Item 3"),
    ]
    .spacing(10)
    .padding(20)
    .into()
}

fn row_layout_example() -> Element<'static, Message> {
    row![
        text("Item 1"),
        text("Item 2"),
        text("Item 3"),
    ]
    .spacing(20)
    .padding(20)
    .into()
}

fn nested_layout_example() -> Element<'static, Message> {
    column![
        text("Column 1"),
        row![
            column![
                text("Nested A1"),
                text("Nested A2"),
            ]
            .spacing(5),
            column![
                text("Nested B1"),
                text("Nested B2"),
            ]
            .spacing(5),
        ]
        .spacing(10),
    ]
    .spacing(10)
    .padding(20)
    .into()
}
