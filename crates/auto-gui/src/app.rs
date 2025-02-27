use gpui::*;
use gpui_widgets::theme::*;

pub struct AutoGuiApp {
}

impl AutoGuiApp {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn run<T: Render>(&self, maker: impl Fn(&mut Context<T>) -> T + 'static) {
        Application::new().run(move |cx: &mut App| {
            init(cx);
            let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            };
            cx.open_window(window_options, |_, cx| cx.new(maker)).unwrap();
        });
    }
}
