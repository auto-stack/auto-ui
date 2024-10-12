use gpui::*;
use autoui::widget::pane::*;
use autoui::widget::toolbar::*;
use autoui::style::theme::ActiveTheme;
use autoui::widget::workspace::Workspace;
use autoui::app::SimpleApp;

struct RootView {
    workspace: View<Workspace>,
}

struct LeftContent {
    text: String,
}

impl Render for LeftContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.text.clone())
            .child(uniform_list(cx.view().clone(), "entries", 10, |_this, range, _cx| {
                let mut items = Vec::new();
                for ix in range {
                    let item = ix + 1;
                    items.push(
                        div()
                            .id(ix)
                            .px_2()
                            .cursor_pointer()
                            .child(format!("Entry_{item}")),
                    );
                }
                items
            }).h_full())
    }
}

struct RightContent {
    text: String,
}

impl Render for RightContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}

struct CenterContent {
    text: String,
}

impl Render for CenterContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}

struct TopContent {
    text: String,
}

impl Render for TopContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}

struct BottomContent {
    text: String,
}

impl Render for BottomContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
       let toolbar = cx.new_view(|_cx| Toolbar{});
       let left = Pane::new(PaneSide::Left, Pixels(250.0)).child(cx.new_view(|_cx| LeftContent { text: "Left".to_string() }));
       let right = Pane::new(PaneSide::Right, Pixels(250.0)).child(cx.new_view(|_cx| RightContent { text: "Right".to_string() }));
       let top = Pane::new(PaneSide::Top, Pixels(100.0)).child(cx.new_view(|_cx| TopContent { text: "Top".to_string() }));
       let bottom = Pane::new(PaneSide::Bottom, Pixels(100.0)).child(cx.new_view(|_cx| BottomContent { text: "Bottom".to_string() }));
       let center = cx.new_view(|_cx| CenterContent { text: "Center".to_string() });
       let workspace = Workspace::new()
        .toolbar(toolbar)
        .left(cx.new_view(|_cx| left))
        .right(cx.new_view(|_cx| right))
        .top(cx.new_view(|_cx| top))
        .bottom(cx.new_view(|_cx| bottom))
        .child(center);
    
       Self {
            workspace: cx.new_view(|_cx| workspace),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.workspace.clone())
    }
}

fn main() {
    SimpleApp::new().run(false, |cx| {
        cx.new_view(|cx| RootView::new(cx))
    });
}


