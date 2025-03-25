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


pub struct LeftPaneStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
}

impl Story for LeftPaneStory {
    fn title() -> &'static str {
        "LeftPane"
    }

    fn description() -> &'static str {
        "LeftPane Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl LeftPaneStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            msg: SharedString::new("LeftPane"),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }


}

impl Focusable for LeftPaneStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LeftPaneStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        col().child(Label::new(self.msg.clone()))
    }
    
}


pub struct RightPaneStory {
    focus_handle: gpui::FocusHandle,
    msg: SharedString,
}

impl Story for RightPaneStory {
    fn title() -> &'static str {
        "RightPane"
    }

    fn description() -> &'static str {
        "RightPane Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl RightPaneStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            msg: SharedString::new("RightPane"),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }


}

impl Focusable for RightPaneStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RightPaneStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        col().child(Label::new(self.msg.clone()))
    }
    
}


pub struct MiddlePaneStory {
    focus_handle: gpui::FocusHandle,
    username: SharedString,
    password: SharedString,
    status: SharedString,
    input_username: Entity<TextInput>,
    input_password: Entity<TextInput>,
}

impl Story for MiddlePaneStory {
    fn title() -> &'static str {
        "MiddlePane"
    }

    fn description() -> &'static str {
        "MiddlePane Example"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl MiddlePaneStory {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            username: SharedString::new("nil"),
            password: SharedString::new("nil"),
            status: SharedString::new(""),
            input_username: cx.new(|cx| TextInput::new(w, cx)),
            input_password: cx.new(|cx| TextInput::new(w, cx)),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn on(&mut self, ev: SharedString) {
        self.status = format!("Login {}", self.username).into();
    }
    
}

impl Focusable for MiddlePaneStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MiddlePaneStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.username = self.input_username.read(cx).text();
        self.password = self.input_password.read(cx).text();
        center()
            .child(
                col()
                    .id("login-story")
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_4()
                    .rounded_lg()
                    .gap_6()
                    .w_2_5()
                    .child(
                        row()
                            .w_begin()
                            .child(
                                v_form()
                                    .child(
                                        form_field()
                                            .label("Username")
                                            .child(self.input_username.clone()),
                                    )
                                    .child(
                                        form_field()
                                            .label("Password")
                                            .child(self.input_password.clone()),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("Login")
                            .label("Login")
                            .on_click(
                                cx
                                    .listener(|v, _, _, cx| {
                                        v.on("button-login".into());
                                        cx.notify();
                                    }),
                            ),
                    )
                    .child(Label::new(self.username.clone()))
                    .child(Label::new(self.status.clone())),
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
        let left_item = DockItem::tab(
            StoryContainer::panel::<LeftPaneStory>(w, cx),
            &dockarea,
            w,
            cx,
        );

        let middle_item = DockItem::tab(
            StoryContainer::panel::<MiddlePaneStory>(w, cx),
            &dockarea,
            w,
            cx,
        );

        let right_item = DockItem::tab(
            StoryContainer::panel::<RightPaneStory>(w, cx),
            &dockarea,
            w,
            cx,
        );



        _ = dockarea.update(cx, |view, cx| {
            view.set_left_dock(left_item, Some(px(350.)), true, w, cx);
            view.set_center(middle_item, w, cx);
            view.set_right_dock(right_item, Some(px(350.)), true, w, cx);
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

        create_new_window_sized("Docks Example", |w, cx| cx.new(|cx| Docks::new(w, cx)), cx, 800, 600);
    });
}
