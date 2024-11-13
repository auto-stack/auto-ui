use crate::spec::WidgetSpec;
use autogui::app::Viewable;
use autogui::widget::button::Button;
use autogui::widget::table::{Align, ColConfig, Row, Table};
use autogui::widget::tab::{TabPane, TabView};
use autolang::ast::{Stmt, Expr, Key, Name};
use autoval::value::Value;
use crate::dyna::app::GlobalSpecState;
use autogui::widget::util::center;
use gpui::*;

pub struct DynaView {
    spec: Option<WidgetSpec>,
    builder: Option<Box<dyn Fn(Div, &mut WidgetSpec, &mut ViewContext<Self>) -> Div + 'static>>,
    kids: Vec<AnyView>,
}

pub struct ViewBuilder {
    pub view_id: ElementId,
    pub builder: Box<dyn Fn(Div, &autolang::ast::Node, &mut WidgetSpec, &mut ViewContext<DynaView>, Option<AnyView>) -> Div + 'static>,
}

impl Viewable for DynaView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        // let mut state = State::new();
        // state.set_int("count", 0);
        cx.observe_global::<GlobalSpecState>(|v, cx| {
            let spec = GlobalSpecState::global(cx);
            let widget_specs = &spec.widget_specs;
            if v.spec.is_some() {
                match v.spec.as_ref().unwrap().id.as_str() {
                    "left" => {
                        v.spec = widget_specs.left.clone();
                    }
                    "right" => {
                        v.spec = widget_specs.right.clone();
                    }
                    "top" => {
                        v.spec = widget_specs.top.clone();
                    }
                    "bottom" => {
                        v.spec = widget_specs.bottom.clone();
                    }
                    "center" => {
                        v.spec = widget_specs.center.clone();
                    }
                    _ => {}
                }
            }
            v.update_spec(cx);
        }).detach();
        Self {
            spec: None,
            builder: None,
            kids: Vec::new(),
        }
    }
}

impl DynaView {
    pub fn reload(&mut self, cx: &mut ViewContext<Self>) {
        if let Some(spec) = self.spec.as_mut() {
            spec.reload();
        }
        self.update_spec(cx);
    }

    pub fn set_spec(&mut self, spec: WidgetSpec) {
        self.spec = Some(spec);
    }

    pub fn from_file(&mut self, path: &str) {
        let spec = WidgetSpec::from_file(path);
        self.set_spec(spec);
    }

    pub fn update_spec(&mut self, cx: &mut ViewContext<Self>) {
        // self.spec.set_state(&mut self.state);

        let spec_view = self.spec.as_ref().unwrap().get_ast_view();
        println!("spec_view: {:?}", spec_view);
        let mut all_builders = Vec::new();
        let mut all_views = Vec::new();
        if let Some(view) = spec_view {
            for (idx, (name, node)) in view.nodes.iter().enumerate() {
                let node = node.clone();
                let (views, builders) = parse_node(&name.text, &node, self.spec.as_mut().unwrap(), idx, cx);
                for view in views.into_iter() {
                    all_views.push(view);
                }
                for builder in builders.into_iter() {
                    all_builders.push(builder);
                }
                // div = parse_node(div, &name.text, &node, spec, cx);
            }
        } 
        for view in all_views.into_iter() {
            self.kids.push(view);
        }
        let spec_view = self.spec.as_ref().unwrap().get_ast_view();
        let kids = self.kids.clone();
        self.builder = Some(Box::new(move |div, spec, cx| {
            let mut div = div;
            if let Some(view) = spec_view.clone() {
                for (i, (_, node)) in view.nodes.iter().enumerate() {
                    let node = node.clone();
                    let builder = all_builders.get(i).unwrap();
                    let view_id = builder.view_id.clone();
                    let builder = &builder.builder;
                    let view = find_view_by_id(&view_id, &kids);
                    div = builder(div, &node, spec, cx, view);
                }
            }
            div
        }));
    }

    pub fn contents(
        mut self,
        builder: impl Fn(Div, &mut WidgetSpec, &mut ViewContext<Self>) -> Div + 'static,
    ) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }

}

fn find_view_by_id(id: &ElementId, views: &Vec<AnyView>) -> Option<AnyView> {
    for view in views.iter() {
        if let Some(view_id) = view.id() {
            if view_id == *id {
                return Some(view.clone());
            }
        }
    }
    None
}

fn parse_node(name: &str, node: &autolang::ast::Node, spec: &mut WidgetSpec, idx: usize, cx: &mut ViewContext<'_, DynaView>) -> (Vec<AnyView> , Vec<ViewBuilder>)  {
    let mut builders: Vec<ViewBuilder> = Vec::new();
    let mut views = Vec::new();
    match name {
        "button" => builders.push(ViewBuilder { view_id: ElementId::Integer(idx), builder: Box::new(add_button) }),
        "text" => builders.push(ViewBuilder { view_id: ElementId::Integer(idx), builder: Box::new(add_text) }),
        "table" => builders.push(ViewBuilder { view_id: ElementId::Integer(idx), builder: Box::new(add_table) }),
        "tabs" => {
            let view = node_view_tabs(&node, spec, idx, cx);
            let id = view.id().unwrap();
            views.push(view);
            builders.push(ViewBuilder { view_id: id, builder: Box::new(add_tabs) });
        }
        _ => {
            // try lookup widget in scope
            println!("lookup widget: {}", name);
            let widget = &spec.scope.borrow().widget;
            match widget {
                Value::Widget(w) => {
                    if w.name == name {
                        // make a new dynamic widget
                        let new_spec = WidgetSpec::new(widget.clone(), ".", "", spec.scope.clone());
                        let mut view = DynaView::new(cx);
                        view.set_spec(new_spec);
                        view.update_spec(cx);
                        let view = cx.new_view(|_cx| view);
                        let id = view.id().unwrap();
                        views.push(view.into());
                        builders.push(ViewBuilder { view_id: id, builder: Box::new(add_widget) });
                        cx.notify();
                    }
                }
                _ => ()
            }
        }
    };
    (views, builders)
}

pub fn add_widget(div: Div, _node: &autolang::ast::Node, _spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>, view: Option<AnyView>) -> Div {
    if let Some(view) = view {
        div.child(view.clone())
    } else {
        div
    }
}

// TODO: currently only support onclick property
fn add_button(mut div: Div, node: &autolang::ast::Node, _spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, _view: Option<AnyView>) -> Div {
    let text_arg = node.args.get(0);
    if let Some(Expr::Str(text)) = text_arg {
        let mut button = Button::primary(text.as_str());

        for stmt in node.body.stmts.iter() {
            if let Stmt::Expr(Expr::Pair(pair)) = stmt {
                match &pair.key {
                    Key::NamedKey(Name { text }) => {
                        if text == "onclick" { 
                            // TODO: remove clone()
                            let v = pair.value.clone();
                            if let Expr::Lambda(lambda) = *v {   
                                button = button.on_click_mut(cx, move |this, _ev, cx| {
                                    this.spec.as_mut().unwrap().run_lambda(&lambda);
                                    cx.notify();
                                });
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        div = div.child(button);
    }
    div
}

fn add_text(mut div: Div, node: &autolang::ast::Node, spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>, _view: Option<AnyView>) -> Div {
    let text_arg = node.args.get(0);
    if let Some(str) = text_arg {
        match str {
            Expr::Str(text) => {
                div = div.child(format!("{}", text));
            }
            Expr::FStr(fstr) => {
                let mut sb = String::new();
                for p in fstr.parts.iter() {
                    match p {
                        Expr::Str(text) => sb.push_str(&format!("{}", text)),
                        Expr::Ident(_) => {
                            let val = spec.eval_ident(p);
                            sb.push_str(&format!("{}", val));
                        }
                        _ => (),
                    }
                }
                div = div.child(sb);
            }
            _ => (),
        }
    }
    div
}

pub fn add_table(mut div: Div, node: &autolang::ast::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, _view: Option<AnyView>) -> Div {
    let config = match node.args.get(0) {
        Some(ident) => spec.eval_expr(&ident),
        None => Value::Nil,
    };
    let config = convert_value_to_table_config(&config);
    let data = match node.args.get(1) {
        Some(ident) => spec.eval_expr(&ident),
        None => Value::Nil,
    };
    let data = convert_value_to_table_data(&data, &config);

    div = div.child(cx.new_view(|cx| Table::new(cx, config, data)));
    div
}

pub fn add_tabs(div: Div, _node: &autolang::ast::Node, _spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>, view: Option<AnyView>) -> Div {
    if let Some(view) = view {
        div.child(view.clone())
    } else {
        div
    }
}


fn node_to_tab(node: &autolang::ast::Node, spec: &mut WidgetSpec, idx: usize, cx: &mut ViewContext<'_, DynaView>) -> View<DynaView> {
    let tab_widget = WidgetSpec::from_ast_node(node, &spec.path, spec.scope.clone());
    let view = cx.new_view(|cx| {
        let mut view = DynaView::new(cx);
        view.set_spec(tab_widget);
        view.update_spec(cx);
        view
    });
    view
}

pub fn node_view_tabs(node: &autolang::ast::Node, spec: &mut WidgetSpec, idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {
    let mut tabs = Vec::new();
    for (idx, stmt) in node.body.stmts.iter().enumerate() {
        match stmt {
            Stmt::Node(node) => {
                let tab_view = node_to_tab(node, spec, idx, cx);
                let name = match node.args.get(0) {
                    Some(Expr::Str(name)) => name,
                    _ => format!("view {}", idx),
                };
                tabs.push((name, tab_view));
            }
            _ => (),
        }
    }
    let len = tabs.len();
    cx.new_view(|cx| {
        let mut tabpane = TabPane::new(cx);
        for tab in tabs.into_iter() {
            tabpane = tabpane.add(cx.new_view(|cx| TabView::new(cx, tab.0, tab.1)));
        }
        if len > 0 {
            tabpane.set_active(0, cx);
        }
        tabpane
    }).into()
}

pub fn convert_value_to_table_config(value: &Value) -> Vec<ColConfig> {
    match value {
        Value::Array(array) => {
            let mut cols = vec![];
            for (idx, item) in array.iter().enumerate() {
                match item {
                    Value::Object(obj) => {
                        let col = ColConfig {
                            idx,
                            id: obj.get_str_or("id", "").into(),
                            title: obj.get_str_or("name", ""),
                            width: obj.get_or("width", Value::Float(0.0)).into(),
                            showas: obj.get_str_or("showas", "text").into(),
                            align: Align::Start,
                        };
                        cols.push(col);
                    }
                    _ => (),
                }
            }
            cols
        }
        _ => vec![],
    }
}

pub fn convert_value_to_table_data(value: &Value, config: &Vec<ColConfig>) -> Vec<Row> {
    match value {
        Value::Array(array) => {
            let mut rows = vec![];
            for item in array.iter() {
                let mut cells = Vec::new();
                for col in config.iter() {
                    match item {
                        Value::Object(obj) => {
                            let cell = obj.get_or(&col.id, Value::Nil);
                            cells.push(cell.clone());
                        }
                        _ => (),
                    }
                }
                rows.push(Row { cells });
            }
            rows
        }
        _ => vec![],
    }
}

impl Render for DynaView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = center();

        if self.builder.is_none() {
            println!("no builder");
            return div;
        }
        let builder = self.builder.take().unwrap();
        let div = builder(div, &mut self.spec.as_mut().unwrap(), cx);
        self.builder = Some(builder);
        div
    }
}

struct StrView {
    text: String,
}   

impl Render for StrView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        center().child(self.text.clone())
    }
}
