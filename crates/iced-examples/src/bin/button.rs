// Button example for iced
use iced::widget::{button, column, text};
use iced::{Center, Element};

fn main() -> iced::Result {
    iced::run(ButtonExample::update, ButtonExample::view)
}

#[derive(Default)]
struct ButtonExample {
    click_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonClicked,
}

impl ButtonExample {
    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonClicked => self.click_count += 1,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("Click the button:"),
            button(text(format!("Clicked {} times", self.click_count)))
                .on_press(Message::ButtonClicked)
        ]
        .padding(20)
        .align_x(Center)
        .into()
    }
}
