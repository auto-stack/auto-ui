use gpui::*;
use autogui::widget::button::*;
use autogui::app::SimpleApp;
use autogui::app::Viewable;
use autogui::style::theme::*;
use autogui::widget::icon::*;

struct ButtonView {
    label: String,
}

impl Viewable for ButtonView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            label: "world".into(),
        }
    }
}

impl Render for ButtonView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let fg_color = theme.secondary_foreground;
        div()
            .flex()
            .flex_col()
            .child(format!("Hello, {}!", self.label))
            .gap_1()
            .child(Button::button("Click Me").on_click_mut(cx, |view, _ev, cx| {
                view.label = "Button".into();
                cx.notify();
            }))
            .child(Button::iconed(SysIcon::Sun.icon().color(fg_color)).on_click_mut(cx, |view, _ev, cx| {
                view.label = "Sun".into();
                Theme::change(ThemeMode::Light, cx);
                cx.notify();
            }))
            .child(Button::iconed(SysIcon::Moon.icon().color(fg_color)).on_click_mut(cx, |view, _ev, cx| {
                view.label = "Moon".into();
                Theme::change(ThemeMode::Dark, cx);
                cx.notify();
            }))
    }
}

fn main() {
    SimpleApp::new().title("Button Example").run_simple::<ButtonView>();
}

