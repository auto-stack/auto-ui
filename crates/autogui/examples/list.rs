use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::button::Button;
use autogui::widget::input::TextInput;
use autogui::widget::list::List;
use autogui::widget::toolbar::*;
use autogui::widget::util::*;
use autogui::widget::workspace::Workspace;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {
    list: View<List>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label_width = 100.;
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .w_1_3()
            .child(
                card("List Demo", cx)
                    .rounded_lg()
                    .shadow_md()
                    .gap_5()
                    .child(self.list.clone())
            )
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            list: cx.new_view(|cx| List::new(cx, vec!["Apple".into(), "Banana".into(), "Cherry".into()])),
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

fn main() {
    SimpleApp::new().run(false, |cx| cx.new_view(|cx| RootView::new(cx)));
}
