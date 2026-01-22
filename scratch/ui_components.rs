// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten

use auto_ui::Component;
use auto_ui::View;

#[derive(Debug)]
pub struct ComponentsDemo {
    pub username: String,
    pub email: String,
    pub agree_terms: bool,
    pub subscribe_newsletter: bool,
    pub plan_choice: i32,
    pub country: String,
}

impl ComponentsDemo {
    pub fn new(username: String, email: String, agree_terms: bool, subscribe_newsletter: bool, plan_choice: i32, country: String) -> Self {
        Self {
            username,
            email,
            agree_terms,
            subscribe_newsletter,
            plan_choice,
            country,
        }
    }
}

impl Component for ComponentsDemo {
    type Msg = i32;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            1 => {
            }
            2 => {
                self.plan_choice = 1;
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().spacing(0).padding(0)
    .child(View::text(&"UI Components Showcase"))
    .child(View::text(&"─────────────────────────"))
    .child(View::text(&"Text Inputs:"))
    .child(View::input("Enter username").build())
    .child(View::input("Enter email").build())
    .child(View::text(&"Checkboxes:"))
    .child(View::checkbox(false, "I agree to the terms"))
    .child(View::checkbox(false, "Subscribe to newsletter"))
    .child(View::text(&"Choose a Plan:"))
    .child(View::radio(true, "Free Plan"))
    .child(View::radio(false, "Pro Plan"))
    .child(View::radio(false, "Enterprise Plan"))
    .child(View::text(&"Country:"))
    .child(View::select(vec!["China"]))
    .child(View::button("Submit", 1))
    .child(View::button("Reset", 2))
    .build()
    }
}


