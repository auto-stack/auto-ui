// Unified Tabs Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates the Tabs (horizontal tab navigation) component.
//
// Run with:
//   cargo run --package unified-tabs --features iced
//   cargo run --package unified-tabs --features gpui

use auto_ui::{Component, View};

#[derive(Debug, Default)]
struct TabsApp {
    selected_tab: usize,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    TabChanged(usize),
}

impl Component for TabsApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::TabChanged(index) => {
                self.selected_tab = index;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::tabs(vec![
            "Home".to_string(),
            "Settings".to_string(),
            "Profile".to_string(),
            "About".to_string(),
        ])
        .selected(self.selected_tab)
        .contents(vec![
            // Home tab content
            View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("Home".to_string()))
                .child(View::text("Welcome to the home tab!".to_string()))
                .build(),
            // Settings tab content
            View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("Settings".to_string()))
                .child(View::text("Configure your preferences here.".to_string()))
                .build(),
            // Profile tab content
            View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("Profile".to_string()))
                .child(View::text("User profile information.".to_string()))
                .build(),
            // About tab content
            View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("About".to_string()))
                .child(View::text("This is a unified tabs example.".to_string()))
                .build(),
        ])
        .on_select(|index| Message::TabChanged(index))
        .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    // The same code works with both backends!
    #[cfg(feature = "iced")]
    {
        return auto_ui_iced::run_app::<TabsApp>();
    }

    #[cfg(feature = "gpui")]
    {
        return auto_ui_gpui::run_app::<TabsApp>("Tabs Example");
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err("No backend enabled. Please enable either 'iced' or 'gpui' feature in Cargo.toml.".into())
    }
}
