use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::tab::{Tab, TabBar};
use autogui::widget::workspace::Workspace;
use gpui::*;
use std::fmt::Display;
use autogui::widget::tab::TabPane;

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

impl Display for ByteOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct CenterContent {
    tabpane: View<TabPane>,
}

impl Render for CenterContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.tabpane.clone())
    }
}

impl RootView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let center = cx.new_view(|cx| CenterContent {
            tabpane: cx.new_view(|cx| TabPane::new(cx))
        });
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
