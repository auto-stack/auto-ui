use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::input::TextInput;
use autogui::widget::toolbar::*;
use autogui::widget::util::*;
use autogui::widget::workspace::Workspace;
use autogui::widget::table::Table;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {
    table: View<Table>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label_width = 100.;
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w_3_4()
            .gap_4()
            .child(self.table.clone())
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            table: cx.new_view(|cx| Table::new(cx)),
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
