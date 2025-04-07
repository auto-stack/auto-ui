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


pub struct HelloStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
    button_label: SharedString,
}

impl Story for HelloStory {
    fn title() -> &'static str {
        "Hello"
    }

    // fn description() -> &'static str {
    //     "Hello Example"
    // }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl HelloStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            msg: SharedString::new(r#"Hello World"#),
            button_label: SharedString::new(r#"Click"#),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        self.msg = format!("Hello Button clicked").into();
    }
    
}

impl Focusable for HelloStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for HelloStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        center()
            .child(
                col()
                    .child(Label::new(self.msg.clone()))
                    .child(
                        Button::new(self.button_label.clone())
                            .label(self.button_label.clone())
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-clicked".into());
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
            StoryContainer::panel::<HelloStory>(w, cx),
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

        create_new_window_sized("Hello Example", |w, cx| cx.new(|cx| Docks::new(w, cx)), cx, 800, 600);
    });
}
