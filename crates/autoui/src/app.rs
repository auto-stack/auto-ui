use crate::assets::Assets;
use crate::theme::{ActiveTheme, init_theme};
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
    pub fn new(title: &str) -> Self {
        Self { app: App::new().with_assets(Assets), title: title.to_string() }
    }

    pub fn run<T>(self, build_root_view: impl FnOnce(&mut WindowContext) -> View<T> + 'static)
    where
        T: 'static + Render,
    {
        self.app.run(move |cx| {
            init_theme(cx);

            // window options
            let window_options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(self.title.into()),
                    // appears_transparent: true,
                    // traffic_light_position: Some(point(px(9.0), px(9.0))),
                    ..Default::default()
                }),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                ..WindowOptions::default()
            };

            cx.open_window(window_options, |cx| {
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
