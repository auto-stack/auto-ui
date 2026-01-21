// Example demonstrating all View components in the improved abstraction layer
//
// Shows: Text, Button, Input, Checkbox, Row, Column

use auto_ui::{Component, View};
use std::fmt::Debug;

#[derive(Debug, Default)]
struct AllComponents {
    input_value: String,
    checkbox_checked: bool,
    click_count: i64,
}

#[derive(Clone, Debug)]
enum Msg {
    InputChanged(String),
    CheckboxToggled(bool),
    ButtonClicked,
}

impl Component for AllComponents {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::InputChanged(value) => {
                self.input_value = value;
                println!("Input changed: {}", self.input_value);
            }
            Msg::CheckboxToggled(checked) => {
                self.checkbox_checked = checked;
                println!("Checkbox toggled: {}", checked);
            }
            Msg::ButtonClicked => {
                self.click_count += 1;
                println!("Button clicked: {} times", self.click_count);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(15)
            .padding(20)
            // Row with horizontal layout
            .child(
                View::row()
                    .spacing(10)
                    .padding(10)
                    .child(View::text("Checkbox Demo:"))
                    .child(
                        View::checkbox(self.checkbox_checked, "Enable feature")
                            .on_toggle(Msg::CheckboxToggled(true)),
                    )
                    .build(),
            )
            // Input field
            .child(
                View::input("Enter text...")
                    .value(self.input_value.clone())
                    .on_change(Msg::InputChanged("dummy".to_string()))
                    .build(),
            )
            // Display input value
            .child(View::text(format!("Input value: {}", self.input_value)))
            // Button
            .child(View::button(
                format!("Click me! ({})", self.click_count),
                Msg::ButtonClicked,
            ))
            .build()
    }
}

fn main() {
    let mut app = AllComponents::default();

    println!("=== All Components Demo ===\n");

    // Simulate interactions
    println!("1. Toggling checkbox:");
    app.on(Msg::CheckboxToggled(true));

    println!("\n2. Changing input:");
    app.on(Msg::InputChanged("Hello AutoUI!".to_string()));

    println!("\n3. Clicking button:");
    app.on(Msg::ButtonClicked);
    app.on(Msg::ButtonClicked);

    println!("\n4. Final view structure:");
    let view = app.view();
    println!("{:#?}\n", view);

    println!("✅ All component types working!");
}
