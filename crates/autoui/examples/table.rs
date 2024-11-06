use autogui::app::SimpleApp;
use autogui::app::Viewable;
use autoui::dyna::dyna::DynaView;
use autogui::widget::workspace::Workspace;
use autogui::widget::toolbar::Toolbar;
use autogui::style::theme::ActiveTheme;
use gpui::*;


pub struct RootView {
    workspace: View<Workspace>,
}

impl RootView {
    pub fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            dyna: cx.new_view(|cx| {
                let mut view = DynaView::new(cx);
                view.set_path("crates/autoui/examples/table.at");
                view.update_spec();
                view
            }),
        });
        let workspace = Workspace::new().toolbar(toolbar).child(center);

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

struct CenterContent {
    dyna: View<DynaView>,
}

impl Render for CenterContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w_3_4()
            .gap_4()
            .child(self.dyna.clone())
    }
}

fn main() {
    println!("current working directory: {}", std::env::current_dir().unwrap().display());
    SimpleApp::new().run(false, |cx| cx.new_view(|cx| RootView::new(cx)));
}
