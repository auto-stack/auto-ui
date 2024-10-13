use gpui::*;
use crate::style::theme::*;
use crate::widget::icon::*;
use crate::style::size::SizeScale;
use gpui::prelude::FluentBuilder;

#[derive(IntoElement)]
pub struct Button {
    icon: Option<Icon>,
    label: Option<SharedString>,
    base: Div,
    style: ButtonStyles,
    size_scale: SizeScale,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
}

struct ButtonStyle {
    bg: Hsla,
    text_color: Hsla,
    hover_color: Hsla,
}

pub enum ButtonStyles {
    Primary,
    Secondary,
    Bare,
}

impl Button {
    // constructors
    pub fn new() -> Self {
        Self {
            icon: None,
            label: None,
            base: div(),
            size_scale: SizeScale::M,
            on_click: None,
            style: ButtonStyles::Secondary,
        }
    }

    pub fn primary(label: &str) -> Self {
        Self::new().label(label.to_string().into()).style(ButtonStyles::Primary)
    }

    pub fn button(label: &str) -> Self {
        Self::new().label(label.to_string().into())
    }

    pub fn iconed(icon: Icon) -> Self {
        Self::new().icon(icon).style(ButtonStyles::Bare)
    }

    // builders
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn label(mut self, label: SharedString) -> Self {
        self.label = Some(label);
        self
    }

    pub fn style(mut self, style: ButtonStyles) -> Self {
        self.style = style;
        self
    }

    pub fn size_scale(mut self, size: SizeScale) -> Self {
        self.size_scale = size;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_click_mut<T: Render>(mut self, cx: &mut ViewContext<T>, handler: impl Fn(&mut T, &ClickEvent, &mut ViewContext<'_, T>) + 'static) -> Self {
        self.on_click = Some(Box::new(cx.listener(move |view, ev, cx| {
            (handler)(view, ev, cx);
        })));
        self
    }

    fn get_style(&self, cx: &mut WindowContext) -> ButtonStyle {
        let theme = cx.active_theme();
        match self.style {
            ButtonStyles::Primary => ButtonStyle {
                bg: theme.primary,
                text_color: theme.primary_foreground,
                hover_color: theme.primary_hover,
            },
            ButtonStyles::Secondary => ButtonStyle {
                bg: theme.secondary,
                text_color: theme.secondary_foreground,
                hover_color: theme.secondary_hover,
            },
            ButtonStyles::Bare => ButtonStyle {
                bg: theme.transparent,
                text_color: theme.secondary_foreground,
                hover_color: theme.secondary_hover,
            },
        }
    }

}


impl RenderOnce for Button {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let style = self.get_style(cx);
        self.base
            .id("button")
            .flex()
            .flex_row()
            .cursor_pointer()
            .bg(style.bg)
            .text_color(style.text_color)
            .hover(|this| this.bg(style.hover_color))
            .rounded_sm()
            .p(px(4.0))
            .items_center()
            .justify_center()
            .overflow_hidden()
            .map(move |this| {
                match self.style {
                    ButtonStyles::Bare => this.size_6(),
                    _ => this.h_8().px_1(),
                }
            })
            .when_some(self.icon, |this, icon| this.child(icon))
            .when_some(self.label, |this, label| this.child(label.clone()))
            .when_some(self.on_click, |this, on_click| {
                this.on_mouse_down(MouseButton::Left, move |_ev, cx| {
                    cx.prevent_default();
                }).on_click(move |ev, cx| {
                    (on_click)(&ev, cx);
                })
            })
    }
}
