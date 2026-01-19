// Dropdown (combo_box) example for iced
use iced::widget::{center, column, combo_box, scrollable, text, space};
use iced::{Element, Fill, Center};
use std::fmt;

fn main() -> iced::Result {
    iced::run(Example::update, Example::view)
}

struct Example {
    language: combo_box::State<Language>,
    selected: Option<Language>,
    text: String
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Selected(Language),
    OptionHovered(Language),
    Closed,
}

impl Example {
    fn new() -> Self {
        Self {
            language: combo_box::State::new(Language::ALL.to_vec()),
            selected: None,
            text: String::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Selected(lan) => {
                self.selected = Some(lan);
                self.text = lan.hello().to_string();
            }
            Message::OptionHovered(lan) => {
                self.text = lan.hello().to_string();
            }
            Message::Closed => {
                self.text = self.selected.map(|l| l.hello().to_string()).unwrap_or_default();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let combo_box = combo_box(
            &self.language,
            "Type a language...",
            self.selected.as_ref(),
            Message::Selected,
        )
        .on_option_hovered(Message::OptionHovered)
        .on_close(Message::Closed)
        .width(250);

        let content = column![
            text(&self.text),
            "What is your language?",
            combo_box,
            space().height(150),
        ]
        .width(Fill)
        .align_x(Center)
        .spacing(10);

        center(scrollable(content)).into()
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
