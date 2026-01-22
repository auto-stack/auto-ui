// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct InputForm {
    pub username: String,
    pub email: String,
    pub message: String,
    pub style: String,
}

impl InputForm {
    pub fn new(username: String, email: String, message: String, style: String) -> Self {
        Self {
            username,
            email,
            message,
            style,
        }
    }
}

impl Component for InputForm {
    type Msg = /* unknown type */;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            "username_changed" => {
            }
            "email_changed" => {
            }
            "message_changed" => {
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&"User Information Form"))
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Username:"))
    .child(View::input("""").value("")
    .build())
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Email:"))
    .child(View::input("""").value("")
    .build())
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Message:"))
    .child(View::input("""").value("")
    .build())
    .build())
    .child(View::col().spacing(0).padding(0)
    .child(View::text(&"Summary:"))
    .child(View::text(&""))
    .child(View::text(&""))
    .child(View::text(&""))
    .build())
    .build()
    }
}


