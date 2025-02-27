use auto_ui::*;
use auto_widgets::*;
pub fn main() {
    AutoApp::new().center(vec![
        Text::new("Hello, World!"),
        Text::new("Hello, Universe!"),
    ]).run();
}
