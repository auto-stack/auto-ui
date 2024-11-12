use crate::widget::pane::*;
use crate::widget::toolbar::*;
use crate::widget::util::*;
use gpui::*;
use prelude::FluentBuilder;
use crate::style::color::Colorize;
use crate::style::theme::ActiveTheme;
use crate::event::ReloadEvent;

pub struct Workspace {
    toolbar: View<Toolbar>,
    left: Option<View<Pane>>,
    right: Option<View<Pane>>,
    top: Option<View<Pane>>,
    bottom: Option<View<Pane>>,
    child: Option<AnyView>,
}

impl EventEmitter<ReloadEvent> for Workspace {}

impl Workspace {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {

        let toolbar = cx.new_view(|_cx| Toolbar {});
        cx.subscribe(&toolbar, |this, _view, _ev, cx| {
            cx.emit(ReloadEvent);
            // tell all the children to reload
        }).detach();
        Self {
            toolbar,
            left: None,
            right: None,
            top: None,
            bottom: None,
            child: None,
        }
    }

    pub fn child(mut self, child: impl Into<AnyView>) -> Self {
        self.child = Some(child.into());
        self
    }

    pub fn left(mut self, left: impl Into<View<Pane>>) -> Self {
        self.left = Some(left.into());
        self
    }

    pub fn right(mut self, right: impl Into<View<Pane>>) -> Self {
        self.right = Some(right.into());
        self
    }

    pub fn top(mut self, top: impl Into<View<Pane>>) -> Self {
        self.top = Some(top.into());
        self
    }

    pub fn bottom(mut self, bottom: impl Into<View<Pane>>) -> Self {
        self.bottom = Some(bottom.into());
        self
    }

}

impl FluentBuilder for Workspace {}

impl Render for Workspace {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        // let status_color = match theme.mode {
        //     ThemeMode::Dark => {
        //         theme.background.lighten(0.05)
        //     }
        //     ThemeMode::Light => {
        //         theme.background.darken(0.05)
        //     }
        // };
        let status_color = theme.background;
        div()
            .flex()
            .flex_col()
            .size_full()
            .border_0()
            // Toolbar
            .child(self.toolbar.clone())
            // Workarea
            .child(
                div()
                    .flex()
                    .flex_row()
                    .size_full()
                    .border_0()
                    // Left Pane
                    .when(self.left.is_some(), |s| {
                        if let Some(left) = self.left.as_ref() {
                            s.child(div().w(left.read(cx).size).h_full().child(left.clone()))
                        } else {
                            s
                        }
                    })
                    // Center Panes
                    .child(
                        col()
                            .size_full()
                            .justify_center()
                            .items_center()
                            .bg(theme.background.darken(0.03))
                            // Center Content
                            .child(self.child.as_ref().unwrap().clone())
                            // Bottom Pane
                            .when(self.bottom.is_some(), |s| {
                                if let Some(bottom) = self.bottom.as_ref() {
                                    s.child(div().h(bottom.read(cx).size).child(bottom.clone()))
                                } else {
                                    s
                                }
                            }),
                    )
                    // Right Pane
                    .when(self.right.is_some(), |s| {
                        if let Some(right) = self.right.as_ref() {
                            s.child(div().w(right.read(cx).size).h_full().child(right.clone()))
                        } else {
                            s
                        }
                    }),
            )
            .child(
                div()
                    .w_full()
                    .h(px(25.0))
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(status_color)
            )
    }
}
