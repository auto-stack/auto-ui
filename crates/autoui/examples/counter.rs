use autogui::app::SimpleApp;
use autogui::app::SimpleRootView;
use autogui::app::Viewable;
use autoui::dyna::simple::*;
use gpui::*;

fn main() {
    println!("current working directory: {}", std::env::current_dir().unwrap().display());
    SimpleApp::new().run(true, |cx| {
        cx.new_view(|cx| SimpleRootView::new(cx.new_view(|cx| {
            let mut view = SimpleDynaView::new(cx);
            view.set_path("crates/autoui/examples/counter.at");
            view.update_spec();
            view
        })))
    });
}
