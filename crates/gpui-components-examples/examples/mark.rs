use auto_ui::*;
use gpui::prelude::FluentBuilder;
use auto_ui::row;
use gpui_component::ActiveTheme;
use markdown::*;
use gpui::{
    Application, App, AppContext, Context, Entity, Focusable, ClickEvent, 
    Render, Window, SharedString, IntoElement, ParentElement,
};

use gpui_component::{
    h_flex,
    input::TextInput,
    button::Button,
    label::Label,
    form::{v_form, form_field}
};

pub struct MarkStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
    button_label: SharedString,
}

impl Story for MarkStory {
    fn title() -> &'static str {
        "Hello"
    }

    fn description() -> &'static str {
        "Hello Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl MarkStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        let markdown_style = MarkdownStyle {
            base_text_style: gpui::TextStyle {
                font_family: "Zed Plex Sans".into(),
                color: cx.theme().colors().terminal_ansi_black,
                ..Default::default()
            },
            code_block: StyleRefinement::default()
                .font_family("Zed Plex Mono")
                // .m(rems(1.))
                .p(rems(1.))
                .bg(rgb(0xEEEEEE)),
            inline_code: gpui::TextStyleRefinement {
                font_family: Some("Zed Mono".into()),
                color: Some(cx.theme().colors().editor_foreground),
                background_color: Some(cx.theme().colors().editor_background),
                ..Default::default()
            },
            rule_color: Color::Muted.color(cx),
            block_quote_border_color: Color::Muted.color(cx),
            block_quote: gpui::TextStyleRefinement {
                color: Some(Color::Default.color(cx)),
                ..Default::default()
            },
            link: gpui::TextStyleRefinement {
                color: Some(Color::Accent.color(cx)),
                underline: Some(gpui::UnderlineStyle {
                    thickness: px(1.),
                    color: Some(Color::Accent.color(cx)),
                    wavy: false,
                }),
                ..Default::default()
            },
            syntax: cx.theme().syntax().clone(),
            selection_background_color: {
                let mut selection = cx.theme().players().local().selection;
                selection.fade_out(0.7);
                selection
            },
            ..Default::default()
        };
        Self {
            focus_handle: cx.focus_handle(),
            msg: SharedString::new("Hello World"),
            button_label: SharedString::new("Click"),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {self.msg=ev}
}

impl Focusable for MarkStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MarkStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center()
            .child(
                col()
                    .child(Label::new(self.msg.clone()))
                    .child(
                        Button::new(self.button_label.clone())
                            .label(self.button_label.clone())
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-clicked".into());
                                        cx.notify();
                                    }),
                            ),
                    ),
            )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Hello Example", StoryView::view::<MarkStory>, cx, 480, 800);
    });
}
