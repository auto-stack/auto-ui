use gpui::*;
use gpui::div;
use gpui::prelude::FluentBuilder as _;
use crate::style::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Checkbox {
    checked: bool,
    disabled: bool,
    id: ElementId,
    on_click: Option<Box<dyn Fn(&bool, &mut WindowContext) + 'static>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { checked: true, disabled: false, id: id.into(), on_click: None }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, onclick: impl Fn(&bool, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(onclick));
        self
    }

    pub fn on_click_mut<T: Render>(mut self, cx: &mut ViewContext<T>, handler: impl Fn(&mut T, &bool, &mut ViewContext<'_, T>) + 'static) -> Self {
        self.on_click = Some(Box::new(cx.listener(move |view, checked, cx| {
            (handler)(view, checked, cx);
        })));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.active_theme();
        let color = if self.disabled { theme.primary.opacity(0.5) } else { theme.primary };
        let icon_color = if self.disabled { theme.primary_foreground.opacity(0.5) } else { theme.primary_foreground };

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .line_height(relative(1.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .relative()
                    .border_1()
                    .border_color(color)
                    .rounded_sm()
                    .size_4()
                    .flex_shrink_0()
                    .map(|this| match self.checked {
                        false => this.bg(theme.transparent),
                        _ => this.bg(color)
                    })
                    .child(
                        svg()
                            .absolute()
                            .top_px()
                            .left_px()
                            .size_3()
                            .text_color(icon_color)
                            .map(|this| match self.checked {
                                true => this.path("icons/check.svg"),
                                _ => this,
                            })
                    )
           )
           .when(self.disabled, |this| {
                this.cursor_not_allowed()
                    .text_color(theme.muted_foreground)
           })
           .when_some(
            self.on_click.filter(|_| !self.disabled),
            |this, onclick| {
                this.on_click(move |_ev, cx| {
                    let checked = !self.checked;
                    onclick(&checked, cx);
                    cx.refresh();
                })
            })
    }
}
