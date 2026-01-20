// Radio example demonstrating single selection from multiple options
//
// This shows how to use radio buttons for choosing between alternatives

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;

#[derive(Debug, Default)]
struct RadioApp {
    selected_language: Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum Language {
    #[default]
    Rust,
    Python,
    JavaScript,
    Go,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    SelectLanguage(Language),
}

impl Component for RadioApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::SelectLanguage(lang) => {
                self.selected_language = lang;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Choose your favorite programming language:".to_string()))
            .child(self.view_language_selection())
            .child(self.view_result())
            .build()
    }
}

impl RadioApp {
    fn view_language_selection(&self) -> View<Message> {
        View::col()
            .spacing(8)
            .child(
                View::radio(self.selected_language == Language::Rust, "Rust")
                    .on_select(Message::SelectLanguage(Language::Rust))
            )
            .child(
                View::radio(self.selected_language == Language::Python, "Python")
                    .on_select(Message::SelectLanguage(Language::Python))
            )
            .child(
                View::radio(self.selected_language == Language::JavaScript, "JavaScript")
                    .on_select(Message::SelectLanguage(Language::JavaScript))
            )
            .child(
                View::radio(self.selected_language == Language::Go, "Go")
                    .on_select(Message::SelectLanguage(Language::Go))
            )
            .build()
    }

    fn view_result(&self) -> View<Message> {
        let lang_name = match self.selected_language {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::Go => "Go",
        };

        View::container(
            View::text(format!("You selected: {}", lang_name))
        )
        .padding(20)
        .width(300)
        .center_x()
        .build()
    }
}

fn main() -> iced::Result {
    iced::run(RadioApp::update, view)
}

fn view(app: &RadioApp) -> iced::Element<'_, Message> {
    app.view_iced()
}
