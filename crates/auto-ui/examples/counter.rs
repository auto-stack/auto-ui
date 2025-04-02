use std::sync::Arc;
use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{
    h_flex,
    ActiveTheme,
    input::TextInput,
    button::Button,
    label::Label,
    dock::{DockArea, DockItem},
    form::{v_form, form_field}
};

use auto_ui::*;
use auto_ui::row;


pub struct CounterStory {
    focus_handle: gpui::FocusHandle,
    count: i32,
}

impl Story for CounterStory {
    fn title() -> &'static str {
        "Counter"
    }

    // fn description() -> &'static str {
    //     "Counter Example"
    // }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl CounterStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            count: 0
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        if ev == "button-inc" {
            self.count = self.count + 1
        } else if ev == "button-dec" {
            self.count = self.count - 1
        }
    }
    
}

impl Focusable for CounterStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CounterStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center()
            .child(
                col()
                    .child(
                        Button::new("+")
                            .label("+")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-inc".into());
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(Label::new(self.count.to_string()))
                    .child(
                        Button::new("-")
                            .label("-")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-dec".into());
                                        cx.notify();
                                    }),
                            ),
                    ),
            )
    }
    
}


pub struct Docks {
    dockarea: Entity<DockArea>,
}

pub struct DockAreaTab {
    id: &'static str,
    version: usize,
}

const MAIN_DOCK_AREA: DockAreaTab = DockAreaTab {
    id: "main-dock",
    version: 5,
};

impl Docks {
    pub fn new(w: &mut Window, cx: &mut Context<Self>) -> Self {
        let dockarea = cx.new(|cx| DockArea::new(
            MAIN_DOCK_AREA.id,
            Some(MAIN_DOCK_AREA.version),
            w,
            cx,
        ));
        let weak_dockarea = dockarea.downgrade();
        Self::layout(&weak_dockarea, w, cx);

        Self {
            dockarea,
        }
    }

    fn layout(dockarea: &WeakEntity<DockArea>, w: &mut Window, cx: &mut Context<Self>) {

        let middle_item = DockItem::tab(
            StoryContainer::panel::<CounterStory>(w, cx),
            &dockarea,
            w,
            cx,
        );




        _ = dockarea.update(cx, |view, cx| {
            view.set_center(middle_item, w, cx);
        })
    }
}

impl Render for Docks {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
        .id("story-workspace")
        .relative()
        .size_full()
        .flex()
        .flex_col()
        .child(self.dockarea.clone())
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Counter Example", |w, cx| cx.new(|cx| Docks::new(w, cx)), cx, 800, 600);
    });
}
