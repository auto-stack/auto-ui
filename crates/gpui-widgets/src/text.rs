use autoval::AutoStr;
use gpui::*;

pub struct TextView {
    text: AutoStr,
}

impl TextView {
    pub fn new(text: impl Into<AutoStr>) -> Self {
        Self { text: text.into() }
    }
}

impl Render for TextView {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let text = SharedString::from(self.text.to_string());
        div()
            .border_1()
            .border_color(rgb(0x00ffff))
            .child(text)
    }
}
