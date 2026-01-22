// Unified Select Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates dropdown selection from multiple options using native widgets.
// Now with callback-based API that receives the selected value!
//
// Run with:
//   cargo run --package unified-select --features iced
//   cargo run --package unified-select --features gpui
//
use auto_ui::{Component, View};

#[derive(Debug, Default)]
struct SelectApp {
    selected_language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}

impl Language {
    fn hello(&self) -> &str {
        match self {
            Language::Chinese => "你好",
            Language::English => "Hello",
        }
    }

    fn name(&self) -> &str {
        match self {
            Language::Chinese => "中文",
            Language::English => "English",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "中文" => Language::Chinese,
            "English" | _ => Language::English,
        }
    }

    fn options() -> Vec<String> {
        vec!["中文".to_string(), "English".to_string()]
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    SelectLanguage(Language),
}

impl Component for SelectApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::SelectLanguage(language) => {
                self.selected_language = language;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Select Example".to_string()))
            .child(View::text(self.selected_language.hello().to_string()))
            .child(View::text("What is your language?".to_string()))
            .child(
                View::select(Language::options())
                    .selected(self.selected_language as usize)
                    .on_choose(|_index, value| {
                        // ✅ Callback receives the selected value!
                        Message::SelectLanguage(Language::from_str(value))
                    })
            )
            .child(View::text("Click the dropdown to select a language".to_string()))
            .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running with Iced backend (using pick_list with callback)");
        return auto_ui_iced::run_app::<SelectApp>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running with GPUI backend (callback support enabled!)");
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
