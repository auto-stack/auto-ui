use gpui::*;
use autoval::AutoStr;
use crate::Center;
use gpui_widgets::TextView;
use auto_widgets::Text;
use auto_adapter::AutoAdapter;


#[derive(Clone)]
pub enum LayoutKind {
    None,
    Center,
}

pub struct AutoGuiApp {
    layout: LayoutKind,
}

type ViewMaker<T> = Box<dyn Fn(&mut Context<T>) -> T + 'static>;

impl AutoGuiApp {
    pub fn new() -> Self {
        Self {
            layout: LayoutKind::None,
        }
    }

    pub fn simple<T: Render>(&mut self, maker: ViewMaker<T>) -> &mut Self {
        self.layout = LayoutKind::None;
        self
    }

    pub fn center() -> Self {
        Self {
            layout: LayoutKind::Center,
        }
    }

    pub fn run<T: Render>(&self, maker: impl Fn(&mut Context<T>) -> T + 'static) {
        let layout = self.layout.clone();
        Application::new().run(move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            };
            match layout {
                LayoutKind::None => {
                    cx.open_window(window_options, |_, cx| cx.new(maker),).unwrap();
                }
                LayoutKind::Center => {
                    cx.open_window(window_options, |_, cx| cx.new(|cx| Center::new(cx.new(maker)))).unwrap();
                }
            }
        });
    }
}
