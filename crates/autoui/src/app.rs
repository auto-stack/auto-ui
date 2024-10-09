use autoui_theme::theme;
use autoui_theme::theme::ActiveTheme;
use gpui::*;

pub struct SimpleApp {
    app: App,
}

pub trait Viewable: 'static + Sized {
    fn view(cx: &mut WindowContext) -> View<Self>;
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
        let theme = cx.theme();
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
            theme::init(cx);

            cx.open_window(WindowOptions::default(), |cx| {
                build_root_view(cx)
            })
            .unwrap();
        });
    }

    pub fn run_simple<T: Viewable + Render>(self) {
        self.run(|cx| {
            let view = T::view(cx);
            cx.new_view(|cx| SimpleRootView::new(view))
        });
    }
}
