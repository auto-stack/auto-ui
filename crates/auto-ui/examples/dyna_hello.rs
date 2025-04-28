use crate::dyna::Dyna;
use crate::dyna::DynaStory;
use auto_ui::*;

use gpui::{
    App, AppContext, Application, Context, Entity, Focusable, IntoElement, ParentElement, Render,
    SharedString, Window,
};

use gpui_component::{button::Button, label::Label};

fn main() {
    let app = Application::new().with_assets(Assets);
    app.run(move |cx| {
        init(cx);
        cx.activate(true);
        create_new_window_sized("Hello Example", StoryView::view::<DynaStory>, cx, 800, 600);
    });
}
