use gpui::*;
use crate::assets::Assets;
use crate::style::theme::{init_theme, ActiveTheme};

pub struct GlobalState {
    pub count: i32,
}

impl Global for GlobalState {}

#[derive(Debug)]
pub struct ReloadState {

}



impl Global for ReloadState {}

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

pub struct SimpleStrView {
    text: SharedString,
}

impl Viewable for SimpleStrView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self { text: "".into() }
    }
}

impl SimpleStrView {
    pub fn text(mut self, text: SharedString) -> Self {
        self.text = text.clone();
        self
    }
}

impl Render for SimpleStrView {
    fn render(&mut self, _cx: &mut ViewContext<'_, Self>) -> impl IntoElement {
        div().child(self.text.clone())
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

// type State = HashMap<String, usize>;

pub struct ElemView {
    builder: Option<Box<dyn Fn(Div) -> Div + 'static>>,
}

impl Viewable for ElemView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self { builder: None }
    }
}

impl ElemView {
    pub fn set_builder(mut self, builder: impl Fn(Div) -> Div + 'static) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }
}

impl Render for ElemView {
    fn render(&mut self, _cx: &mut ViewContext<'_, Self>) -> impl IntoElement {
        self.builder.as_ref().unwrap()(div())
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

            let (title_options, window_bounds) = if is_simple {
                (
                    TitlebarOptions {
                        title: Some(self.title.into()),
                        ..Default::default()
                    },
                    Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(320.0), px(240.0)),
                        cx,
                    ))),
                )
            } else {
                (
                    TitlebarOptions {
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(9.0), px(9.0))),
                        ..Default::default()
                    },
                    None,
                )
            };

            // window options
            let window_options = WindowOptions {
                titlebar: Some(title_options),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                window_bounds,
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

    pub fn run_text(self, text: &str) {
        let shared_text: SharedString = text.to_string().into();
        self.run(true, move |cx| {
            let view = cx.new_view(|_cx| SimpleStrView::new(_cx).text(shared_text));
            cx.new_view(|_cx| SimpleRootView::new(view))
        });
    }

    pub fn run_elems(self, builder: impl Fn(Div) -> Div + 'static) {
        self.run(true, |cx| {
            cx.new_view(|cx| SimpleRootView::new(cx.new_view(|cx| {
                ElemView::new(cx).set_builder(builder)
            })))
        });
    }
}
