use gpui::*;
use autoval::value::Value;
use autoval::value::Node;
use autogui::assets::Assets;
use autogui::style::theme::{init_theme, ActiveTheme};
use autogui::app::Viewable;
use autogui::widget::workspace::Workspace;
use autogui::widget::toolbar::Toolbar;
use autogui::widget::pane::PaneSide;
use autogui::widget::pane::Pane;
use autogui::widget::button::Button;
use autoval::value::Widget;
use autoval::value::MetaID;
use autoval::value::Model;
use crate::spec::{Spec, WidgetSpec};
use crate::dyna::dyna::DynaView;

pub struct RootView {
    workspace: View<Workspace>,
}

impl RootView {
    pub fn new(cx: &mut ViewContext<Self>, workspace: View<Workspace>) -> Self {
        Self {
            workspace,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.workspace.clone())
    }
}

struct DynaContent {
    dyna: View<DynaView>,
}

impl Render for DynaContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .w_3_4()
            .gap_4()
            .child(self.dyna.clone())
    }
}

pub struct DynaApp {
    app: App,
    path: String,
}

impl DynaApp {
    pub fn new(path: &str) -> Self {
        Self {
            app: App::new().with_assets(Assets),
            path: path.to_string(),
        }
    }

    pub fn run(self) {
        self.app.run(move |cx| {
            init_theme(cx);

            let title_options = TitlebarOptions {
                appears_transparent: true,
                traffic_light_position: Some(point(px(9.0), px(9.0))),
                ..Default::default()
            };

            // window options
            let window_options = WindowOptions {
                titlebar: Some(title_options),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                ..WindowOptions::default()
            };

            cx.open_window(window_options, |cx| cx.new_view(|cx: &mut ViewContext<RootView>| {

                let spec = Spec::from_file(&self.path);
                // check for `app` block
                let app_spec = spec.get_app_node();
                if app_spec.is_none() {
                    panic!("app spec not found in {}", self.path);
                }
                let app_spec = app_spec.unwrap();

                // Prepare workspace
                let workspace_view = cx.new_view(|cx| {
                    let mut workspace = Workspace::new(cx);
                    let path = &self.path.clone();

                    for sub in app_spec.nodes.iter() {
                        match sub.name.as_str() {
                            "center" => {
                                workspace = Self::create_widget(workspace, PaneSide::Center, &spec, sub, path, cx);
                            }
                            "bottom" => {
                                workspace = Self::create_widget(workspace, PaneSide::Bottom, &spec, sub, path, cx);
                            }
                            "left" => {
                                workspace = Self::create_widget(workspace, PaneSide::Left, &spec, sub, path, cx);
                            }
                            "right" => {
                                workspace = Self::create_widget(workspace, PaneSide::Right, &spec, sub, path, cx);
                            }
                            "top" => {
                                workspace = Self::create_widget(workspace, PaneSide::Top, &spec, sub, path, cx);
                            }
                            _ => {
                                panic!("unknown block: {}", sub.name);
                            }
                        }
                    }
                    workspace
                });
                RootView::new(cx, workspace_view)
            }))
            .unwrap();

        });
    }

    fn create_widget(workspace: Workspace, side: PaneSide, spec: &Spec, block: &Node, path: &str, cx: &mut ViewContext<Workspace>) -> Workspace {
        println!("create_widget: {}", block);
        // let mut array_view = ArrayView::new();
        // look for block's nodes:
        // let widget_spec = if block.nodes.len() == 1 && block.nodes[0].name == "service_table" { 
            // println!("service_table");
            // spec.get_widget().clone()
        // } else {
            // node_to_widget(block)
        // };
        let widget_spec = node_to_widget(block);

        let view = cx.new_view(|cx| DynaContent {
            dyna: cx.new_view(|cx| {
                let widget_spec = WidgetSpec::new(widget_spec, path, spec.scope_shared());
                let mut view = DynaView::new(cx);
                view.set_spec(widget_spec);
                view.update_spec();
                view
            }),
        });
        
        match side {
            PaneSide::Center => workspace.child(view),
            PaneSide::Bottom => workspace.bottom(cx.new_view(|cx| Pane::new(side, Pixels(250.0)).child(view))),
            PaneSide::Left => workspace.left(cx.new_view(|cx| Pane::new(side, Pixels(250.0)).child(view))),
            PaneSide::Right => workspace.right(cx.new_view(|cx| Pane::new(side, Pixels(250.0)).child(view))),
            PaneSide::Top => workspace.top(cx.new_view(|cx| Pane::new(side, Pixels(250.0)).child(view))),
        }
    }

}

fn node_to_widget(block: &Node) -> Value {
    let node_body_id = block.body.clone();
    Value::Widget(Widget { name: block.name.clone(), model: Model::new(), view_id: node_body_id })
}

struct StrView {
    text: String,
}

impl Render for StrView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(self.text.clone())
    }
}


struct ArrayView {
    array: Vec<AnyView>,
}

impl ArrayView {
    pub fn new() -> Self {
        Self { array: vec![] }
    }

    pub fn push(&mut self, view: AnyView) {
        self.array.push(view);
    }

}

impl Render for ArrayView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .children(self.array.clone())
    }
}
