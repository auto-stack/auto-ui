// Checkbox example for iced
use iced::widget::{center, checkbox, column, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run(Example::update, Example::view)
}

#[derive(Default)]
struct Example {
    good: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    FeelingGood(bool),
}

impl Example {
    fn update(&mut self, message: Message) {
        match message {
            Message::FeelingGood(good) => {
                self.good = good;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let default_checkbox = checkbox(self.good)
            .label("Feeling Good?")
            .on_toggle(Message::FeelingGood);

        let msg = if self.good {
            "I'm feeling GOOD!"
        } else {
            "I'm feeling bad ..."
        };

        let col = column![default_checkbox, text(msg).size(40)].padding(40);

        center(col).into()
    }
}
