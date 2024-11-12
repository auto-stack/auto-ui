use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::workspace::Workspace;
use gpui::*;
use std::fmt::Display;
use autogui::widget::tab::{TabPane, TabView};
use autogui::widget::util::center;

struct View1 {
    text: String,
}

struct View2 {
    text: String,
}

impl Render for View1 {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        center().child(self.text.clone())
    }
}

impl Render for View2 {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        center().child(self.text.clone())
    }
}


struct RootView {
    workspace: View<Workspace>,
}

#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    Motorola = 0,
    Intel = 1,
}

impl From<usize> for ByteOrder {
    fn from(value: usize) -> Self {
        match value {
            0 => ByteOrder::Motorola,
            1 => ByteOrder::Intel,
            _ => panic!("Invalid byte order value"),
        }
    }
}

impl RootView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let center = cx.new_view(|cx| TabPane::new(cx)
            .add(cx.new_view(|cx| {
                    let view1 = cx.new_view(|_cx| View1 { text: "View A1".to_string() });
                    TabView::new(cx, "View 1", view1)
                }))
                .add(cx.new_view(|cx| {
                    let view2 = cx.new_view(|_cx| View2 { text: "View A2".to_string() });
                    TabView::new(cx, "View 2", view2)
                }))
        );

        let workspace = cx.new_view(|cx| Workspace::new(cx).child(center));

        Self {
            workspace,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.workspace.clone())
    }
}

fn main() {
    SimpleApp::new().run(false, |cx| cx.new_view(|cx| RootView::new(cx)));
}
