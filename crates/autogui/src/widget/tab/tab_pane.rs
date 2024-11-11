use gpui::*;
use super::TabView;
use super::TabBar;
use super::Tab;
use crate::style::theme::ActiveTheme;
use crate::widget::util::*;


pub struct TabPane {
    focus_handle: FocusHandle,
    tab_views: Vec<View<TabView>>,
    active: usize,
}

impl FocusableView for TabPane {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.active_tab().focus_handle(cx)
    }
}

impl TabPane {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let dummy_view_1 = cx.new_view(|cx| TabView::new(cx, "dummy 1"));
        let dummy_view_2 = cx.new_view(|cx| TabView::new(cx, "dummy 2"));
        Self { focus_handle, tab_views: vec![dummy_view_1, dummy_view_2], active: 1 }
    }

    pub fn active_tab(&self) -> &View<TabView> {
        &self.tab_views[self.active]
    }

    pub fn set_active(&mut self, index: usize, cx: &mut ViewContext<Self>) {
        self.active = index;
        self.focus_active(cx);
        cx.notify();
    }

    pub fn focus_active(&mut self, cx: &mut ViewContext<Self>) {
        self.active_tab().focus_handle(cx).focus(cx);
    }
}

impl TabPane {
    fn render_tab_bar(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        row()
            .w_full()
            .child(
                TabBar::new("tabbar").children(self.tab_views.iter().enumerate().map(|(i, v)| {
                    let title = v.read(cx).title.clone();
                    Tab::new(SharedString::from(format!("tab-{}", i)), title)
                        .py_2()
                        .selected(i == self.active)
                        .on_click(cx.listener(move |v, _e, cx| v.set_active(i, cx)))
                }))
            )
    }

    fn render_active_view(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("tab-view")
            .group("")
            .overflow_y_scroll()
            .overflow_x_hidden()
            .flex_1()
            .child(self.active_tab().clone())
    }
}


impl Render for  TabPane {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);
        let theme = cx.active_theme();
        col()
            .id("tab-pane")
            .track_focus(&focus_handle)
            .size_full()
            .overflow_hidden()
            .bg(theme.background)
            .child(self.render_tab_bar(cx))
            .child(self.render_active_view(cx))
    }
}