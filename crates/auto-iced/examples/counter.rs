use iced::Center;
use iced::widget::button;
use iced::widget::text;
use iced::widget::column;
use iced::widget::Column;
use auto_val::Obj;

#[derive(Default)]
struct AutoModel {
    obj: Obj
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Inc,
    Dec,
    Reset,
}

impl AutoModel {
    fn update(&mut self, message: Message) {
        match message {
            Message::Inc => self.obj.inc("value"),
            Message::Dec => self.obj.dec("value"),
            Message::Reset => self.obj.reset("value"),
        };
    }

    fn view(&self) -> Column<Message> {
        column![
            button("+").on_press(Message::Inc),
            text(self.obj.get_int_of("value")).size(50),
            button("-").on_press(Message::Dec),
            button("Reset").on_press(Message::Reset),
        ]
        .padding(20)
        .align_x(Center)
    }
}

pub fn main() -> iced::Result {
    println!("Hello, world!");

    iced::run("A cool counter", AutoModel::update, AutoModel::view)
}

