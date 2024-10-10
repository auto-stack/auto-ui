use crate::theme::{ActiveTheme, init_theme};
use gpui::*;

pub struct SimpleApp {
    app: App,
}

pub trait Viewable: 'static + Sized {
    fn new(cx: &mut ViewContext<Self>) -> Self;
}

pub struct SimpleRootView<T: Viewable + Render> {
    view: View<T>,
}

impl<T: Viewable + Render> SimpleRootView<T> {
    pub fn new(view: View<T>) -> Self {
        Self { view }
    }
}

impl<T: Viewable + Render> Render for SimpleRootView<T> {
    fn render(&mut self, cx: &mut ViewContext<'_, Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .flex()
            .bg(theme.background)
            .size_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(theme.border)
            .text_xl()
            .text_color(theme.foreground)
            .child(self.view.clone())
    }
}

impl SimpleApp {
    pub fn new() -> Self {
        Self { app: App::new() }
    }

    pub fn run<T>(self, build_root_view: impl FnOnce(&mut WindowContext) -> View<T> + 'static)
    where
        T: 'static + Render,
    {
        self.app.run(move |cx| {
            init_theme(cx);

            cx.open_window(WindowOptions::default(), |cx| {
                build_root_view(cx)
            })
            .unwrap();
        });
    }

    pub fn run_simple<T: Viewable + Render>(self) {
        self.run(|cx| {
            let view = cx.new_view(|cx| T::new(cx));
            cx.new_view(|_cx| SimpleRootView::new(view))
        });
    }
}
