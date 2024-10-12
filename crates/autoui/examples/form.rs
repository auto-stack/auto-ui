use autoui::app::SimpleApp;
use autoui::style::theme::ActiveTheme;
use autoui::widget::button::Button;
use autoui::widget::card::Card;
use autoui::widget::checkbox::Checkbox;
use autoui::widget::toolbar::*;
use autoui::widget::workspace::Workspace;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {}

impl Render for CenterContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label_width = 100.;
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(Card::new("Section_1")
                .child(
                    div().flex().flex_row()
                        .child(div().w(Pixels(label_width)).child("IsSigned: "))
                        .child(Checkbox::new("is_signed")))
                .child(Button::button("Don't click me")))
            .child(Card::new("Section_2")
                .child("World")
                .child(Button::button("Don't click me")))
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|_cx| CenterContent {});
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
