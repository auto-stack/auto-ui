// Minimal test to check if basic Button works
use auto_ui::{Component, View};

#[derive(Debug, Default)]
struct TestApp {
    counter: usize,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Increment,
}

impl Component for TestApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => {
                self.counter += 1;
                println!("Counter: {}", self.counter);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Simple Button Test".to_string()))
            .child(View::button(format!("Click me: {}", self.counter), Message::Increment))
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "gpui")]
    {
        return auto_ui_gpui::run_app::<TestApp>("Button Test");
    }

    #[cfg(not(feature = "gpui"))]
    {
        Err("GPUI feature not enabled".into())
    }
}
