use gpui::*;
use gpui_story::Story;
// use super::story::Story;
// use gpui::{
//     div, AnyView, App, AppContext, Context, Entity, IntoElement, ParentElement, Render,
//     Styled as _, Window,
// };


pub struct StoryView {
    root: AnyView,
}

impl StoryView {
    pub fn new<T: Story>(window: &mut Window, cx: &mut App) -> Self {
        let entity = T::new_view(window, cx);

        Self {
            root: entity.into(),
        }
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
