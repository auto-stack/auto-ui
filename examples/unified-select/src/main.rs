// Unified Select Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates dropdown selection from multiple options.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run --package unified-select --features iced
//   cargo run --package unified-select --features gpui
//
use auto_ui::{Component, View};

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

// Unified main() - works with BOTH backends!
fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running with Iced backend");
        return auto_ui_iced::run_app::<SelectApp>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running with GPUI backend (with auto-conversion!)");
        return auto_ui_gpui::run_app::<SelectApp>("Select Demo - AutoUI");
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err(
            "❌ No backend enabled!


             Please run with a backend feature:

             • cargo run --features iced

             • cargo run --features gpui"
                .into(),
        )
    }
}

