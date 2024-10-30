use crate::widget::util::*;
use crate::style::theme::ActiveTheme;
use gpui::*;
use gpui::prelude::FluentBuilder;

pub struct List {
    focus_handle: FocusHandle,
    items: Vec<SharedString>,
    selected: Option<usize>,
    on_selected: Option<Box<dyn Fn(&usize, &mut WindowContext) + 'static>>,
}

impl FocusableView for List {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub enum ListEvent {
    Selected(usize),
}

impl EventEmitter<ListEvent> for List {}

impl List {
    pub fn new(cx: &mut WindowContext, items: Vec<SharedString>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            items,
            selected: None,
            on_selected: None,
        }
    }

    pub fn select(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn unselect(mut self) -> Self {
        self.selected = None;
        self
    }

    pub fn update_selected(&mut self, index: usize) {
        self.selected = Some(index);
    }

    pub fn on_selected(mut self, on_selected: impl Fn(&usize, &mut WindowContext) + 'static) -> Self {
        self.on_selected = Some(Box::new(on_selected));
        self
    }

    pub fn selected_text(&self) -> SharedString {
        if let Some(i) = self.selected {
            self.items[i].clone()
        } else {
            "--".into()
        }
    }
}

impl Render for List {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let size = self.items.len();
        let selected = self.selected;
        col()
            .id("drop-list")
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .max_h(px(500.0))
            .child(
                uniform_list(
                    cx.view().clone(),
                    "uniform-list",
                    size,
                    move |list, range, cx| {
                        range
                            .map(|i| {
                                div()
                                    .id("drop-item")
                                    .px_2()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.active_theme().list_hover))
                                    .w_full()
                                    .when(selected == Some(i), |s| s.bg(cx.active_theme().selection))
                                    .child(list.items[i].clone())

                                    .on_click(cx.listener(move |this, _ev, cx| {
                                        this.selected = Some(i);
                                        cx.emit(ListEvent::Selected(i));
                                        cx.notify();
                                    }))
                            })
                            .collect()
                    },
                )
                .flex_grow()
                .with_sizing_behavior(ListSizingBehavior::Infer),
            )
    }
}
