use crate::widget::pane::*;
use gpui::*;
use prelude::FluentBuilder;

pub struct Workspace {
    left: Option<View<Pane>>,
    right: Option<View<Pane>>,
    top: Option<View<Pane>>,
    bottom: Option<View<Pane>>,
    child: Option<AnyView>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
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
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .flex_col()
                    // Center Content
                    .child(div().size_full().p_4().child(self.child.as_ref().unwrap().clone()))
                    // Bottom Pane
                    .when(self.bottom.is_some(), |s| {
                        if let Some(bottom) = self.bottom.as_ref() {
                            s.child(div().w_full().h(bottom.read(cx).size).child(bottom.clone()))
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
            })
    }
}
