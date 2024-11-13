use autogui::app::SimpleApp;
use autogui::app::Viewable;
use autogui::style::theme::ActiveTheme;
use autogui::widget::dropzone::DropZone;
use autogui::widget::icon::SysIcon;
use autogui::widget::util::center;
use gpui::*;

struct DropFileView {
    focus_handle: FocusHandle,
    dropzone: View<DropZone>,
}

impl Viewable for DropFileView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        DropFileView {
            focus_handle: cx.focus_handle(),
            dropzone: cx.new_view(|cx| {
                DropZone::new(cx).on_drop(Some(Box::new(move |path, _cx| {
                    println!("drop: {:?}", path);
                })))
            }),
        }
    }
}

impl FocusableView for DropFileView {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DropFileView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        center()
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .child(self.dropzone.clone())
            
    }
}

fn main() {
    SimpleApp::new()
        .title("Hello World Example")
        .run_simple::<DropFileView>();
}
