// Select example demonstrating dropdown selection
//
// This shows how to use select dropdowns for choosing from multiple options

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;

#[derive(Debug, Default)]
struct SelectApp {
    selected_theme: Theme,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum Theme {
    #[default]
    Light,
    Dark,
    Auto,
    HighContrast,
}

impl Theme {
    fn all() -> [Theme; 4] {
        [Theme::Light, Theme::Dark, Theme::Auto, Theme::HighContrast]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::Auto => "Auto (System)",
            Theme::HighContrast => "High Contrast",
        }
    }

    fn options() -> Vec<String> {
        Theme::all().iter().map(|t| t.as_str().to_string()).collect()
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    SelectTheme(Theme),
}

impl Component for SelectApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::SelectTheme(theme) => {
                self.selected_theme = theme;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Settings".to_string()))
            .child(self.view_theme_selector())
            .child(self.view_result())
            .build()
    }
}

impl SelectApp {
    fn view_theme_selector(&self) -> View<Message> {
        View::col()
            .spacing(8)
            .child(View::text("Theme:".to_string()))
            .child(
                View::select(Theme::options())
                    .selected(self.selected_theme as usize)
                    .on_choose(Message::SelectTheme(Theme::Light))
            )
            .build()
    }

    fn view_result(&self) -> View<Message> {
        View::container(
            View::text(format!("Selected theme: {}", self.selected_theme.as_str()))
        )
        .padding(20)
        .width(300)
        .center_x()
        .build()
    }
}

fn main() -> iced::Result {
    iced::run(SelectApp::update, view)
}

fn view(app: &SelectApp) -> iced::Element<'_, Message> {
    app.view_iced()
}
