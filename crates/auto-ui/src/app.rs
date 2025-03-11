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

use super::Story;
use gpui::{Window, App, AppContext, Entity, Render, div, Styled as _, ParentElement, AnyView, IntoElement, Context};

pub struct StoryView {
    root: AnyView,
}

impl StoryView {
    pub fn new<T: Story>(window: &mut Window, cx: &mut App) -> Self {
        let entity = T::new_view(window, cx);

        Self { root: entity.into() }
    }

    pub fn view<T: Story>(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new::<T>(window, cx))
    }
}

impl Render for StoryView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_4().size_full().child(self.root.clone())
    }
}




