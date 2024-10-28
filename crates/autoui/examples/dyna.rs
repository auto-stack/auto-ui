use autogui::app::SimpleApp;
use autogui::app::SimpleRootView;
use autogui::app::Viewable;
use autoui::dyna::view::*;
use autoui::spec::*;
use gpui::*;

fn main() {
    let source = r#"
    widget counter {
        model {
            var count = 0
        }
        view {
            button("+") {
                onclick: || count = count + 1
            }
            text("Count: {}")
        }
    }
    "#;
    SimpleApp::new().run(true, |cx| {
        cx.new_view(|cx| SimpleRootView::new(cx.new_view(|cx| {
            let mut spec = Spec::new();
            spec.read_str(source);
            let mut view = DynaView::new(cx);
            view.update_spec(spec);
            view
        })))
    });
}
