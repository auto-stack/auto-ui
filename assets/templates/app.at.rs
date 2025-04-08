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

$ for embed in embeds {
$embed
$ }

$ for story in storys {

pub struct ${story.name}Story {
    focus_handle: gpui::FocusHandle,
$ for f in story.fields {
    ${f.name}: ${f.kind},
$ }
}

impl Story for ${story.name}Story {
    fn title() -> &'static str {
        "${story.name}"
    }

    // fn description() -> &'static str {
    //     "${story.name} Example"
    // }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl ${story.name}Story {
    pub(crate) fn new(w: &mut Window, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        $ for f in story.fields {
            ${f.init_code}
        $ }
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

$ for m in story.methods {
    ${m}
$ }
}

impl Focusable for ${story.name}Story {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ${story.name}Story {
    ${story.code}
}

$ }

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
        $ if app.left {
        let left_item = DockItem::tab(
            StoryContainer::panel::<${app.left}Story>(w, cx),
            &dockarea,
            w,
            cx,
        );
        $ }

        $ if app.middle {
        let middle_item = DockItem::tab(
            StoryContainer::panel::<${app.middle}Story>(w, cx),
            &dockarea,
            w,
            cx,
        );
        $ }

        $ if app.right {
        let right_item = DockItem::tab(
            StoryContainer::panel::<${app.right}Story>(w, cx),
            &dockarea,
            w,
            cx,
        );
        $ }

        $ if app.bottom {
        let bottom_item = DockItem::tab(
            StoryContainer::panel::<${app.bottom}Story>(w, cx),
            &dockarea,
            w,
            cx,
        );
        $ }

        $ if app.top {
        let top_item = DockItem::tab(
            StoryContainer::panel::<${app.top}Story>(w, cx),
            &dockarea,
            w,
            cx,
        );
        $ }

        _ = dockarea.update(cx, |view, cx| {
        $ if app.left {
            view.set_left_dock(left_item, Some(px(350.)), true, w, cx);
        $ }
        $ if app.middle {
            view.set_center(middle_item, w, cx);
        $ }
        $ if app.right {
            view.set_right_dock(right_item, Some(px(350.)), true, w, cx);
        $ }
        $ if app.bottom {
            view.set_bottom_dock(bottom_item, Some(px(350.)), true, w, cx);
        $ }
        $ if app.top {
            view.set_top_dock(top_item, Some(px(350.)), true, w, cx);
        $ }
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

        create_new_window_sized("${app.title}", |w, cx| cx.new(|cx| Docks::new(w, cx)), cx, 800, 600);
    });
}
