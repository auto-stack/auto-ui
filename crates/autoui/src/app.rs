use crate::assets::Assets;
use crate::style::theme::{init_theme, ActiveTheme};
use gpui::*;

pub struct SimpleApp {
    app: App,
    title: String,
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

pub struct HelloView {
    label: String,
}

impl Viewable for HelloView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self { label: "world".into() }
    }
}

impl Render for HelloView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(format!("Hello, {}!", self.label))
    }
}

impl<T: Viewable + Render> Render for SimpleRootView<T> {
    fn render(&mut self, cx: &mut ViewContext<'_, Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .id("Root")
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
        Self {
            app: App::new().with_assets(Assets),
            title: String::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn run<T>(
        self,
        is_simple: bool,
        build_root_view: impl FnOnce(&mut WindowContext) -> View<T> + 'static,
    ) where
        T: 'static + Render,
    {
        self.app.run(move |cx| {
            init_theme(cx);

            let title_options = if is_simple {
                TitlebarOptions {
                    title: Some(self.title.into()),
                    ..Default::default()
                }
            } else {
                TitlebarOptions {
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                    ..Default::default()
                }
            };

            // window options
            let window_options = WindowOptions {
                titlebar: Some(title_options),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                ..WindowOptions::default()
            };

            cx.open_window(window_options, |cx| build_root_view(cx))
                .unwrap();
        });
    }

    pub fn run_simple<T: Viewable + Render>(self) {
        self.run(true, |cx| {
            let view = cx.new_view(|cx| T::new(cx));
            cx.new_view(|_cx| SimpleRootView::new(view))
        });
    }

    pub fn run_hello(self) {
        self.run(false, |cx| {
            let view = cx.new_view(|cx| HelloView::new(cx));
            cx.new_view(|_cx| SimpleRootView::new(view))
        }); 
    }
}
