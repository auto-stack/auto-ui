use gpui::*;
use autoui_theme::theme::*;

#[derive(IntoElement)]
pub struct Button {
    text: String,
    base: Div,
    style: ButtonStyles,
    onclick: Box<dyn Fn(&MouseDownEvent, &mut WindowContext) + 'static>,
}

struct ButtonStyle {
    bg: Hsla,
    text_color: Hsla,
    hover_color: Hsla,
}

enum ButtonStyles {
    Primary,
    Secondary,
    Destructive,
}

impl Button {
    pub fn new(text: String) -> Self {
        Self {
            text,
            base: div(),
            onclick: Box::new(|_, _| {}),
            style: ButtonStyles::Primary,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&MouseDownEvent, &mut WindowContext) + 'static) -> Self {
        self.onclick = Box::new(handler);
        self
    }

    fn is_digit(&self) -> bool {
        self.text.chars().all(|c| c.is_digit(10))
    }

    fn style(&mut self, style: ButtonStyles) -> &mut Self {
        self.style = style;
        self
    }

    fn get_style(&self, cx: &mut WindowContext) -> ButtonStyle {
        match self.style {
            ButtonStyles::Primary => ButtonStyle {
                bg: cx.theme().primary,
                text_color: cx.theme().primary_foreground,
                hover_color: cx.theme().primary_hover,
            },
            ButtonStyles::Secondary => ButtonStyle {
                bg: cx.theme().secondary,
                text_color: cx.theme().secondary_foreground,
                hover_color: cx.theme().secondary_hover,
            },
            ButtonStyles::Destructive => ButtonStyle {
                bg: cx.theme().destructive,
                text_color: cx.theme().destructive_foreground,
                hover_color: cx.theme().destructive_hover,
            },
        }
    }

}


impl RenderOnce for Button {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let style = self.get_style(cx);
        self.base
            .flex()
            .cursor_pointer()
            .bg(style.bg)
            .text_color(style.text_color)
            .hover(|this| this.bg(style.hover_color))
            .rounded_sm()
            .p(px(4.0))
            .items_center()
            .justify_center()
            .child(self.text)
            .on_mouse_down(MouseButton::Left, move |event, ctx| {
                (self.onclick)(event, ctx);
            })
    }
}
