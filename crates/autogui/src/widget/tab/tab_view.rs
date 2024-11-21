use gpui::*;

pub struct TabView {
    focus_handle: FocusHandle,
    pub name: String,
    pub title: String,
    view: AnyView,
}


impl TabView {
    pub fn new(cx: &mut ViewContext<Self>, name: impl Into<String>, title: impl Into<String>, view: impl Into<AnyView>) -> Self {
        let focus_handle = cx.focus_handle();
        Self { focus_handle, name: name.into(), title: title.into(), view: view.into() }
    }
}

impl FocusableView for TabView {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TabView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_start()
            .p_4()
            .track_focus(&focus_handle)
            .child(self.view.clone())
    }
}