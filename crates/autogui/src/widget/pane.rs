use gpui::*;
use prelude::FluentBuilder;
use crate::style::color::Colorize;
use crate::style::theme::{ActiveTheme, ThemeMode};


pub const PANE_MIN_SIZE: Pixels = Pixels(100.);

#[derive(Clone, Debug)]
pub enum PaneSide {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

pub struct Pane {
    // size of the pane, e.g. A LeftPane with size 200px has a width of 200px
    pub size: Pixels,
    hidden: bool,
    side: PaneSide,
    child: Option<AnyView>,
}

impl Pane {
    pub fn new(side: PaneSide, size: Pixels) -> Self {
        Self {
            side,
            size,
            hidden: false,
            child: None,
        }
    }

    pub fn child(mut self, child: impl Into<AnyView>) -> Self {
        self.child = Some(child.into());
        self
    }

    pub fn is_vertical(&self) -> bool {
        matches!(self.side, PaneSide::Top | PaneSide::Bottom)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self.side, PaneSide::Left | PaneSide::Right)
    }

    pub fn toggle(mut self) -> Self {
        self.hidden = !self.hidden;
        self
    }

}

impl FluentBuilder for Pane {}

impl Render for Pane {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let bg_color = match theme.mode {
            ThemeMode::Dark => theme.background.lighten(0.03),
            ThemeMode::Light => theme.background.darken(0.03),
        };
        div()
            .flex()
            .flex_grow()
            .size_full()
            .relative()
            .p_2()
            .map(|s| {
                match self.side {
                    PaneSide::Left => s.border_r_1(),
                    PaneSide::Right => s.border_l_1(),
                    PaneSide::Top => s.border_b_1(),
                    PaneSide::Bottom => s.border_t_1(),
                    PaneSide::Center => s,
                }
            })
            .bg(bg_color)
            .border_color(theme.border)
            .map(|s| {
                if self.size.is_zero() {
                    s.flex_shrink()
                } else {
                    s.flex_basis(self.size)
                }
            })
            .when(self.is_vertical(), |s| s.min_h(PANE_MIN_SIZE))
            .when(self.is_horizontal(), |s| s.min_w(PANE_MIN_SIZE))
            .when_some(self.child.clone(), |s, c| s.child(c))
    }
}
