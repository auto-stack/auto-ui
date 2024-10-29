use autogui::app::SimpleApp;
use autogui::app::SimpleRootView;
use autogui::app::Viewable;
use autoui::dyna::view::*;
use autoui::spec::*;
use gpui::*;

fn main() {
    println!("current working directory: {}", std::env::current_dir().unwrap().display());
    SimpleApp::new().run(true, |cx| {
        cx.new_view(|cx| SimpleRootView::new(cx.new_view(|cx| {
            let mut spec = Spec::new();
            spec.read_file("counter.au");
            let mut view = DynaView::new(cx);
            view.update_spec(spec);
            view
        })))
    });
}
