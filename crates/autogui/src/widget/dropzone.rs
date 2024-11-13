use gpui::*;
use crate::style::theme::ActiveTheme;
use crate::widget::icon::SysIcon;
use crate::widget::util::center;
use gpui::prelude::FluentBuilder;

pub struct DropZone {
    focus_handle: FocusHandle,
    message: SharedString,
    path: String,
    pub ondrop: Option<Box<dyn Fn(&str, &mut ViewContext<Self>) + 'static>>,
}


impl DropZone {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            message: "Drop a File here".into(),
            path: String::new(),
            ondrop: None,
        }
    }

    pub fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub fn on_drop(mut self, ondrop: impl Fn(&str, &mut ViewContext<Self>) + 'static) -> Self {
        self.ondrop = Some(Box::new(ondrop));
        self
    }
}

impl Render for DropZone {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        center()
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .flex_none()
            .bg(theme.background)
            .border_2()
            .border_color(theme.border)
            .on_drag_move::<ExternalPaths>(cx.listener(move |_this, ev: &DragMoveEvent<ExternalPaths>, _cx| {
                println!("drag move: {:?}", ev.event.position);
            }))
            .child(SysIcon::Inbox.icon().size(Rems(2.)))
            .text_size(Rems(1.))
            .child(self.message.clone())
            .child(
                div()
                    .id("dropzone")
                    .invisible()
                    .absolute()
                    .size_full()
                    .bg(theme.drop_target)
                    .drag_over::<ExternalPaths>(|style, _ev, cx| {
                        style.visible().border_2().border_color(cx.active_theme().drag_border)
                    })
                    .on_drop(cx.listener(move |this, paths: &ExternalPaths, cx| {
                        this.path = paths.paths().first().unwrap().display().to_string();
                        this.message = format!("{}", this.path).into();
                        if let Some(ondrop) = this.ondrop.as_ref() {
                            ondrop(&this.path, cx);
                        }
                        cx.notify();
                    }))
            )
    }
}