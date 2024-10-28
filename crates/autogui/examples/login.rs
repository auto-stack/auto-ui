use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::button::Button;
use autogui::widget::input::TextInput;
use autogui::widget::theme_toggle::ThemeToggle;
use autogui::widget::toolbar::*;
use autogui::widget::util::*;
use autogui::widget::workspace::Workspace;
use gpui::*;

struct RootView {
    workspace: View<Workspace>,
}

struct CenterContent {
    name_input: View<TextInput>,
    password_input: View<TextInput>,
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
                card("Login to your account", cx)
                    .rounded_lg()
                    .shadow_md()
                    .gap_5()
                    .child(
                        row()
                            .w_full()
                            .child(div().w(Pixels(label_width)).child("Name: "))
                            .child(div().w_full().child(self.name_input.clone())),
                    )
                    .child(
                        row()
                            .w_full()
                            .child(div().w(Pixels(label_width)).child("Password: "))
                            .child(div().w_full().child(self.password_input.clone())),
                    )
                    .child(
                        row()
                            .w_full()
                            .gap_5()
                            .child(Button::primary("Login").on_click(|_ev, cx| {
                                println!("clicked login");
                                cx.refresh();
                            }))
                            .child(div().flex_grow())
                            .child(Button::button("Cancel").on_click(|_ev, cx| {
                                println!("clicked cancel");
                                cx.refresh();
                            }))
                    ),
            )
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            name_input: cx.new_view(|cx| TextInput::new(cx)),
            password_input: cx.new_view(|cx| TextInput::new(cx)),
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
