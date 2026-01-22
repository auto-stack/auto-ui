// Select (pick_list) example for iced
use iced::widget::{center, column, pick_list, text, Space};
use iced::{Element, Fill, Center};
use std::fmt;

fn main() -> iced::Result {
    iced::run(Example::update, Example::view)
}

struct Example {
    selected: Option<Language>,
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Selected(Language),
}

impl Example {
    fn new() -> Self {
        Self {
            selected: Some(Language::English),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Selected(language) => {
                self.selected = Some(language);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let greeting = self.selected.map(|l| l.hello().to_string()).unwrap_or("Select a language".to_string());

        let pick_list = pick_list(
            &Language::ALL[..],
            self.selected,
            Message::Selected,
        )
        .placeholder("Select a language")
        .width(250);

        let content = column![
            text("Select Example").size(32),
            text(greeting).size(48),
            text("What is your language?").size(20),
            pick_list,
            text("Click the dropdown to select a language").size(14),
            Space::new().height(150.0),
        ]
        .width(Fill)
        .align_x(Center)
        .spacing(20);

        center(content).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}

impl Language {
    const ALL: [Language; 2] = [Language::Chinese, Language::English];

    fn hello(&self) -> &str {
        match self {
            Language::Chinese => "你好",
            Language::English => "Hello",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Language::Chinese => "中文",
            Language::English => "English",
        })
    }
}
