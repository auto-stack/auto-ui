use gpui::*;
use autoval::value::Value;
use autoval::value::Node;
use autogui::assets::Assets;
use autogui::style::theme::{init_theme, ActiveTheme};
use autogui::app::Viewable;
use autogui::widget::workspace::Workspace;
use autogui::widget::pane::PaneSide;
use autogui::widget::pane::Pane;
use autoval::value::Widget;
use autoval::value::Model;
use crate::spec::{Spec, WidgetSpec};
use crate::dyna::dyna::DynaView;
use gpui::ReadGlobal;
use autogui::app::{GlobalState, ReloadState};
use std::rc::Rc;
use std::cell::RefCell;
use autolang::scope::Universe;

pub struct RootView {
    workspace: View<Workspace>,
}

impl RootView {
    pub fn new(_cx: &mut ViewContext<Self>, workspace: View<Workspace>) -> Self {
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

pub struct WidgetSpecs {
    pub left: Option<WidgetSpec>,
    pub right: Option<WidgetSpec>,
    pub top: Option<WidgetSpec>,
    pub bottom: Option<WidgetSpec>,
    pub center: Option<WidgetSpec>,
}

impl WidgetSpecs {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
            top: None,
            bottom: None,
            center: None,
        }
    }
}

pub struct GlobalSpecState {
    spec: Spec,
    path: String,
    pub widget_specs: WidgetSpecs,
}

impl Global for GlobalSpecState {}

impl GlobalSpecState {
    pub fn new(path: &str) -> Self {
        let spec = Spec::from_file(path);
        let widget_specs = Self::parse_sides(&spec, path);
        Self {
            spec,
            path: path.to_string(),
            widget_specs,
        }
    }

    fn make_widget_spec(path: &str, id: &str, scope: Rc<RefCell<Universe>>, sub: &Node) -> WidgetSpec {
        WidgetSpec::new(node_to_widget(sub), path, id, scope)
    }

    pub fn parse_sides(spec: &Spec, path: &str) -> WidgetSpecs {
        let app_spec = spec.get_app_node();
        let scope = spec.scope_shared();
        if app_spec.is_none() {
            panic!("app spec not found in {}", path);
        }
        let app_spec = app_spec.unwrap();
        let mut widget_specs = WidgetSpecs::new();
        for sub in app_spec.nodes.iter() {

            match sub.name.as_str() {
                "center" => {
                    widget_specs.center = Some(Self::make_widget_spec(path, "center", scope.clone(), sub));
                }
                "bottom" => {
                    widget_specs.bottom = Some(Self::make_widget_spec(path, "bottom", scope.clone(), sub));
                }
                "left" => {
                    widget_specs.left = Some(Self::make_widget_spec(path, "left", scope.clone(), sub));
                }
                "right" => {
                    widget_specs.right = Some(Self::make_widget_spec(path, "right", scope.clone(), sub));
                }
                "top" => {
                    widget_specs.top = Some(Self::make_widget_spec(path, "top", scope.clone(), sub));
                }
                _ => {
                    panic!("unknown block: {}", sub.name);
                }
            }
        }
        widget_specs
    }

    pub fn reload(&mut self) {
        self.spec.reload();
        self.widget_specs = Self::parse_sides(&self.spec, &self.path);
    }

    pub fn scope(&self) -> Rc<RefCell<Universe>> {
        self.spec.scope_shared()
    }

    pub fn path(&self) -> &str {
        &self.path
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

            let global = GlobalState { count: 0 };
            cx.set_global(global);

            let reload = ReloadState { };
            cx.set_global(reload);



            cx.observe_global::<GlobalState>(|g| {
                println!("global changed: {}", GlobalState::global(g).count);
            }).detach();

            cx.observe_global::<ReloadState>(|cx| {
                println!("reload changed: {:?}", ReloadState::global(cx));
                cx.refresh();
            }).detach();

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

            cx.observe_global::<ReloadState>(|cx| {
                println!("reload changed: {:?}", ReloadState::global(cx));
                // self.spec.reload();

                GlobalSpecState::update_global(cx, |g, _cx| {
                    g.reload();
                });
                // g.notify();
            }).detach();

            cx.open_window(window_options, |cx| cx.new_view(|cx: &mut ViewContext<RootView>| {
                let global_spec = GlobalSpecState::new(&self.path);
                // Prepare workspace
                let workspace_view = cx.new_view(|cx| {
                    let mut workspace = Workspace::new(cx);

                    if let Some(left) = global_spec.widget_specs.left.clone() {
                        workspace = Self::create_widget(workspace, PaneSide::Left, left, cx);
                    }
                    if let Some(right) = global_spec.widget_specs.right.clone() {
                        workspace = Self::create_widget(workspace, PaneSide::Right, right, cx);
                    }
                    if let Some(top) = global_spec.widget_specs.top.clone() {
                        workspace = Self::create_widget(workspace, PaneSide::Top, top, cx);
                    }
                    if let Some(bottom) = global_spec.widget_specs.bottom.clone() {
                        workspace = Self::create_widget(workspace, PaneSide::Bottom, bottom, cx);
                    }
                    if let Some(center) = global_spec.widget_specs.center.clone() {
                        workspace = Self::create_widget(workspace, PaneSide::Center, center, cx);
                    }
                    workspace
                });
                cx.set_global(global_spec);
                RootView::new(cx, workspace_view)
            }))
            .unwrap();

        });
    }

    fn create_widget(workspace: Workspace, side: PaneSide, widget_spec: WidgetSpec, cx: &mut ViewContext<Workspace>) -> Workspace {

        let view = cx.new_view(|cx| {
            let mut view = DynaView::new(cx);
            view.set_spec(widget_spec);
            view.update_spec(cx);
            view
        });
        
        match side {
            PaneSide::Center => workspace.child(view),
            PaneSide::Bottom => workspace.bottom(cx.new_view(|_cx| Pane::new(side, Pixels(150.0)).child(view))),
            PaneSide::Left => workspace.left(cx.new_view(|_cx| Pane::new(side, Pixels(150.0)).child(view))),
            PaneSide::Right => workspace.right(cx.new_view(|_cx| Pane::new(side, Pixels(150.0)).child(view))),
            PaneSide::Top => workspace.top(cx.new_view(|_cx| Pane::new(side, Pixels(150.0)).child(view))),
        }
    }

}

fn node_to_widget(block: &Node) -> Value {
    let node_body_id = block.body.clone();
    Value::Widget(Widget { name: block.name.clone(), model: Model::new(), view_id: node_body_id })
}
