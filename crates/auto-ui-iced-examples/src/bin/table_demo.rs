// Table example demonstrating structured data display
//
// This shows how to use tables for displaying tabular data

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;

#[derive(Debug)]
struct TableApp {
    selected_example: Example,
}

impl Default for TableApp {
    fn default() -> Self {
        Self {
            selected_example: Example::Simple,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Example {
    Simple,
    WithStats,
    Complex,
}

#[derive(Clone, Debug)]
enum Message {
    ShowExample(Example),
}

impl Component for TableApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ShowExample(example) => {
                self.selected_example = example;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Table Examples".to_string()))
            .child(self.view_navigation())
            .child(self.view_current_example())
            .build()
    }
}

impl TableApp {
    fn view_navigation(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button("Simple", Message::ShowExample(Example::Simple)))
            .child(View::button("With Stats", Message::ShowExample(Example::WithStats)))
            .child(View::button("Complex", Message::ShowExample(Example::Complex)))
            .build()
    }

    fn view_current_example(&self) -> View<Message> {
        match self.selected_example {
            Example::Simple => self.view_simple_table(),
            Example::WithStats => self.view_stats_table(),
            Example::Complex => self.view_complex_table(),
        }
    }

    fn view_simple_table(&self) -> View<Message> {
        let headers = vec![
            View::text("Name".to_string()),
            View::text("Age".to_string()),
            View::text("City".to_string()),
        ];

        let rows = vec![
            vec![
                View::text("Alice".to_string()),
                View::text("25".to_string()),
                View::text("New York".to_string()),
            ],
            vec![
                View::text("Bob".to_string()),
                View::text("30".to_string()),
                View::text("London".to_string()),
            ],
            vec![
                View::text("Charlie".to_string()),
                View::text("28".to_string()),
                View::text("Tokyo".to_string()),
            ],
        ];

        View::table(headers, rows)
            .spacing(8)
            .col_spacing(16)
            .build()
    }

    fn view_stats_table(&self) -> View<Message> {
        let headers = vec![
            View::text("Product".to_string()),
            View::text("Sales".to_string()),
            View::text("Revenue".to_string()),
            View::text("Growth".to_string()),
        ];

        let rows = vec![
            vec![
                View::text("Widget A".to_string()),
                View::text("1,234".to_string()),
                View::text("$45,678".to_string()),
                View::text("+12%".to_string()),
            ],
            vec![
                View::text("Widget B".to_string()),
                View::text("2,345".to_string()),
                View::text("$67,890".to_string()),
                View::text("+8%".to_string()),
            ],
            vec![
                View::text("Widget C".to_string()),
                View::text("987".to_string()),
                View::text("$34,567".to_string()),
                View::text("-3%".to_string()),
            ],
            vec![
                View::text("Widget D".to_string()),
                View::text("3,456".to_string()),
                View::text("$89,012".to_string()),
                View::text("+15%".to_string()),
            ],
        ];

        View::scrollable(
            View::table(headers, rows)
                .spacing(8)
                .col_spacing(20)
                .build()
        )
        .height(300)
        .build()
    }

    fn view_complex_table(&self) -> View<Message> {
        let headers = vec![
            View::text("Task".to_string()),
            View::text("Status".to_string()),
            View::text("Priority".to_string()),
            View::text("Due Date".to_string()),
        ];

        let rows = vec![
            vec![
                View::text("Design UI".to_string()),
                View::text("✓ Done".to_string()),
                View::text("High".to_string()),
                View::text("2024-01-15".to_string()),
            ],
            vec![
                View::text("Implement API".to_string()),
                View::text("⏳ In Progress".to_string()),
                View::text("High".to_string()),
                View::text("2024-01-20".to_string()),
            ],
            vec![
                View::text("Write Tests".to_string()),
                View::text("⏳ In Progress".to_string()),
                View::text("Medium".to_string()),
                View::text("2024-01-22".to_string()),
            ],
            vec![
                View::text("Documentation".to_string()),
                View::text("○ Pending".to_string()),
                View::text("Low".to_string()),
                View::text("2024-01-25".to_string()),
            ],
            vec![
                View::text("Deploy to Prod".to_string()),
                View::text("○ Pending".to_string()),
                View::text("High".to_string()),
                View::text("2024-01-28".to_string()),
            ],
        ];

        View::col()
            .spacing(16)
            .child(View::text("Project Tasks".to_string()))
            .child(
                View::scrollable(
                    View::table(headers, rows)
                        .spacing(8)
                        .col_spacing(20)
                        .build()
                )
                .height(350)
                .build()
            )
            .build()
    }
}

fn main() -> iced::Result {
    iced::run(TableApp::update, view)
}

fn view(app: &TableApp) -> iced::Element<'_, Message> {
    app.view_iced()
}
