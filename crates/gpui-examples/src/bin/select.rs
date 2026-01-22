// Select example for gpui-component
// Demonstrates a dropdown selection UI using gpui_component::select::Select
use gpui::*;
use gpui_component::{select::*, Root, *};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    fn hello(&self) -> &str {
        match self {
            Language::Chinese => "你好",
            Language::English => "Hello",
        }
    }

    fn name(&self) -> &str {
        match self {
            Language::Chinese => "中文",
            Language::English => "English",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "Chinese" => Language::Chinese,
            "English" => Language::English,
            _ => Language::English,
        }
    }
}

impl SelectItem for Language {
    type Value = &'static str;

    fn title(&self) -> SharedString {
        self.name().to_string().into()
    }

    fn value(&self) -> &Self::Value {
        match self {
            Language::Chinese => &"Chinese",
            Language::English => &"English",
        }
    }
}

pub struct SelectExample {
    select_state: Entity<SelectState<Vec<Language>>>,
    selected_language: Language,
}

impl SelectExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let languages = vec![Language::Chinese, Language::English];
        let select_state = cx.new(|cx| {
            SelectState::new(
                languages,
                Some(IndexPath::default().row(1)), // Default to English
                window,
                cx,
            )
        });

        // Subscribe to selection changes
        cx.subscribe_in(&select_state, window, Self::on_select_event)
            .detach();

        Self {
            select_state,
            selected_language: Language::English,
        }
    }

    fn on_select_event(
        &mut self,
        _: &Entity<SelectState<Vec<Language>>>,
        event: &SelectEvent<Vec<Language>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(value) => {
                if let Some(lang_value) = value {
                    self.selected_language = Language::from_str(*lang_value);
                }
            }
        }
    }
}

impl Render for SelectExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let greeting = self.selected_language.hello().to_string();

        div()
            .v_flex()
            .gap_6()
            .p_6()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Select Example")
            )
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .child(greeting)
            )
            .child(
                div()
                    .text_lg()
                    .child("What is your language?")
            )
            .child(
                div()
                    .w(px(250.0))
                    .child(Select::new(&self.select_state).placeholder("Select a language"))
            )
            .child(
                div()
                    .text_sm()
                    .child("Click the dropdown to select a language")
            )
    }
}

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(100.0), y: px(100.0) },
                        size: gpui::Size { width: px(600.0), height: px(500.0) },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Select - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| SelectExample::new(window, cx));
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
