// use auto_widgets::*;
// use gpui_widgets::*;
// use auto_gui::*;

// // when using gpui as base
// pub struct AutoApp {
//     pane: Pane,
//     base: AutoGuiApp,
// }

// impl AutoApp {
//     pub fn new() -> Self {
//         let pane = Pane::Empty;
//         let base = AutoGuiApp::new();
//         Self { pane, base }
//     }

//     pub fn center(mut self, texts: Vec<Text>) -> Self {
//         self.pane = Pane::Center(texts.into_iter().map(|text| Kid::Widget(Widget::Text(text))).collect());
//         self
//     }

//     pub fn run(&self) {
//         let pane = self.pane.clone();
//         self.base.run(move |_cx| PaneView::new(pane));
//     }
// }
