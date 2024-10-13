use autoui::app::SimpleApp;
use autoui::style::theme::ActiveTheme;
use autoui::widget::button::Button;
use autoui::widget::checkbox::Checkbox;
use autoui::widget::input::TextInput;
use autoui::widget::toolbar::*;
use autoui::widget::workspace::Workspace;
use autoui::widget::util::*;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {
    input: View<TextInput>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label_width = 100.;
        div()
            .flex()
            .flex_col()
            .w_1_3()
            .items_center()
            .gap_4()
            .child(
                card("Section_1", cx)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .child(div().w(Pixels(label_width)).child("IsSigned: "))
                            .child(Checkbox::new("is_signed")),
                    )
                    .child(Button::button("Don't click me")),
            )
            .child(
                card("Section_2", cx)
                    .child("World")
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .child(self.input.clone())
                    )
                    .child(Button::button("Don't click me")),
            )
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            input: cx.new_view(|cx| TextInput::new(cx)),
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
