use auto_ui::*;
use gpui::prelude::FluentBuilder;
use auto_ui::row;
use gpui_component::ActiveTheme;
use gpui_component::dock::{DockArea, DockItem};
use std::sync::Arc;

use gpui::{
    div, Application, Styled, App, AppContext, Context, Entity, Focusable, ClickEvent, 
    InteractiveElement, IntoElement, ParentElement, Render, Window,
    SharedString, WeakEntity, Axis, px,
};

use gpui_component::{
    h_flex,
    input::TextInput,
    button::Button,
    form::{v_form, form_field}
};

pub struct Workspace {
    // title_bar: Entity<AppTitleBar>,
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

impl Workspace {
    pub fn new(w: &mut Window, cx: &mut Context<Self>) -> Self {
        let dockarea = cx.new(|cx| DockArea::new(
            MAIN_DOCK_AREA.id,
            Some(MAIN_DOCK_AREA.version),
            w,
            cx,
        ));
        let weak_dockarea = dockarea.downgrade();
        Self::layout(&weak_dockarea, w, cx);
        // let title_bar = cx.new(|cx| {
            // AppTitleBar::new("Examples", w, cx)
        // });

        Self {
            // title_bar,
            dockarea,
        }
    }

    fn layout(dockarea: &WeakEntity<DockArea>, w: &mut Window, cx: &mut Context<Self>) {
        // let item = DockItem::tabs(
        //     vec![Arc::new(StoryContainer::panel::<PaneStory>(w, cx))],
        //     None,
        //     &dockarea,
        //     w,
        //     cx,
        // );

        let item = DockItem::tab(
            StoryContainer::panel::<PaneStory>(w, cx),
            &dockarea,
            w,
            cx,
        );

        let left_item = item.clone();
        _ = dockarea.update(cx, |view, cx| {
            view.set_left_dock(left_item, Some(px(350.)), true, w, cx);
            view.set_center(item, w, cx);
        })
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
        .id("story-workspace")
        .relative()
        .size_full()
        .flex()
        .flex_col()
        // .child(self.title_bar.clone())
        .child(self.dockarea.clone())
    }
}

pub struct PaneStory {
    focus_handle: gpui::FocusHandle,
    name_input: Entity<TextInput>,
    password_input: Entity<TextInput>,
    status: SharedString,
}

impl Story for PaneStory {
    fn title() -> &'static str {
        "Login"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl PaneStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            name_input: cx.new(|cx| TextInput::new(w, cx)),
            password_input: cx.new(|cx| TextInput::new(w, cx)),
            status: SharedString::default(),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn on_cancel(_ev: &ClickEvent, _w: &mut Window, _cx: &mut App) {
        println!("Cancel");
    }
}

impl Focusable for PaneStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PaneStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        row().center().child(
            col()
            .id("login-story")
            .border_1()
            .border_color(cx.theme().border)
            .p_4()
            .rounded_lg()
            .gap_6()
            .w_2_5()
            .child(
                row().w_begin().child(
                    v_form()
                        .child(
                            form_field()
                                .label("Name: ")
                                .child(self.name_input.clone()),
                        )
                        .child(
                            form_field()
                                .label("Password: ")
                                .child(self.password_input.clone()),
                        )
                    ))
            .child(
                h_flex()
                    .w_full()
                    .gap_5()
                    .child(Button::new("login").label("Login").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.status = SharedString::from("Logging in...");
                            println!("Username: {}", this.name_input.read(cx).text());
                        })
                    ))
                    .child(div().flex_grow())
                    .child(Button::new("cancel").label("Cancel").on_click(Self::on_cancel))
            )
            .when(!self.status.is_empty(), |e| e.child(self.status.clone())))
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Pane Example", |w, cx| cx.new(|cx| Workspace::new(w, cx)), cx, 800, 600);
    });
}
