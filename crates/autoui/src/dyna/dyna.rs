use crate::dyna::app::GlobalSpecState;
use gpui::rgb;
use gpui::UpdateGlobal;
use crate::spec::WidgetSpec;
use autogui::app::Viewable;
use autogui::app::GlobalDataStoreCollectAction;
use autogui::widget::button::Button;
use autogui::widget::tab::{TabPane, TabView};
use autogui::widget::dropdown::Dropdown;
use autogui::widget::dropzone::DropZone;
use autogui::widget::input::TextInput;
use autogui::widget::table::{Align, ColConfig, Table};
use autogui::widget::util::{col, row};
use autogui::widget::list::List;
use autogui::widget::pane::PaneSide;
use autolang::ast::{Node, Expr, Key, Name, Stmt, Arg};
use autoval::{ValueKey, Value, Grid};
use gpui::{Div, SharedString, ViewContext, View, AnyView, ElementId, Render, IntoElement};
use gpui::prelude::*;


pub struct DynaView {
    compact: bool,
    spec: Option<WidgetSpec>,
    side: PaneSide,
    builder: Option<Box<dyn Fn(Div, &mut WidgetSpec, &mut ViewContext<Self>) -> Div + 'static>>,
}

pub enum DynaComponentView {
    None,
    View(AnyView),
    Subs(Vec<DynaComponent>),
}

pub struct DynaComponent {
    pub view: DynaComponentView,
    pub builder: Box<
        dyn Fn(
                Div,
                &Node,
                &mut WidgetSpec,
                &mut ViewContext<DynaView>,
                &DynaComponentView,
            ) -> Div
            + 'static,
    >,
}

impl DynaComponent {
    pub fn unknown_node(_name: &str) -> Self {
        Self {
            view: DynaComponentView::None,
            builder: Box::new(|div, node, _spec, _cx, _view| {
                div.child(format!("unknown node: {}", node.name.text))
            }),
        }
    }
}

impl Viewable for DynaView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            spec: None,
            compact: false,
            builder: None,
            side: PaneSide::Center,
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

    pub fn set_side(&mut self, side: PaneSide) {
        self.side = side;
    }

    pub fn set_compact(&mut self, compact: bool) {
        self.compact = compact;
    }

    pub fn update_spec(&mut self, cx: &mut ViewContext<Self>) {

        // TODO: should first evaluate all node bodies into REAL nodes
        // Here the nodes in the view are not evaluated yet, their sub nodes are actually Expr::Calls
        // So we need to parse them into real nodes first

        let spec_view = self.spec.as_ref().unwrap().get_ast_view();
        let mut all_components = Vec::new();
        if let Some(view) = spec_view {
            for (idx, (name, node)) in view.nodes.iter().enumerate() {
                let node = node.clone();
                let builder = parse_node(&name.text, &node, self.spec.as_mut().unwrap(), idx, cx);
                all_components.push(builder);
            }
            // Eval body
            for stmt in view.body.stmts.iter() {
                let val = self.spec.as_mut().unwrap().eval_stmt(stmt);
                println!("Value: {}", val);
            }
        }
        let spec_view = self.spec.as_ref().unwrap().get_ast_view();
        self.builder = Some(Box::new(move |div, spec, cx| {
            let mut div = div;
            if let Some(view) = spec_view.clone() {
                for (i, (_, node)) in view.nodes.iter().enumerate() {
                    let node = node.clone();
                    let compo = all_components.get(i).unwrap();
                    let builder = &compo.builder;
                    let view = &compo.view;
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

fn parse_node(name: &str, node: &Node, spec: &mut WidgetSpec, idx: usize, cx: &mut ViewContext<'_, DynaView>) -> DynaComponent {
    match name {
        "button" => DynaComponent {
            view: DynaComponentView::None,
            builder: Box::new(add_button),
        },
        "text" => DynaComponent {
            view: DynaComponentView::None,
            builder: Box::new(add_text),
        },
        "list" => DynaComponent {
            view: DynaComponentView::None,
            builder: Box::new(add_list),
        },
        "table" => {
            let view = add_table_view(&node, spec, idx, cx);
            DynaComponent {
                view: DynaComponentView::View(view),
                builder: Box::new(add_view_clone),
            }
        }
        "input" => {
            let view = add_input_view(&node, spec, idx, cx);
            DynaComponent {
                view: DynaComponentView::View(view),
                builder: Box::new(add_view_clone),
            }
        }
        "tabs" => {
            let view = node_view_tabs(&node, spec, idx, cx);
            DynaComponent {
                view: DynaComponentView::View(view),
                builder: Box::new(add_view_clone),
            }
        }
        "row" => {
            let view = node_view_with_subs(&node, spec, idx, cx);
            DynaComponent {
                view,
                builder: Box::new(add_row),
            }
        }
        "col" => {
            let view = node_view_with_subs(&node, spec, idx, cx);
            DynaComponent {
                view,
                builder: Box::new(add_col),
            }
        }
        "dropzone" => {
            let view = node_view_dropzone(&node, spec, idx, cx);
            DynaComponent {
                view: DynaComponentView::View(view),
                builder: Box::new(add_view_clone),
            }
        }
        "dropdown" => {
            let view = node_view_dropdown(&node, spec, idx, cx);
            DynaComponent {
                view: DynaComponentView::View(view),
                builder: Box::new(add_view_clone),
            }
        }
        _ => {
            // try lookup widget in scope
            println!("lookup widget: {}", name);
            let widget = &spec.scope.borrow().widget();
            match widget {
                Value::Widget(w) => {
                    if w.name == name {
                        // make a new dynamic widget
                        let new_spec = WidgetSpec::new(widget.clone(), ".", "", spec.scope.clone());
                        let mut view = DynaView::new(cx);
                        view.set_spec(new_spec);
                        view.update_spec(cx);
                        let view = cx.new_view(|_cx| view);
                        DynaComponent {
                            view: DynaComponentView::View(view.into()),
                            builder: Box::new(add_view_clone),
                        }
                    } else {
                        println!("widget not found: {}", name);
                        DynaComponent::unknown_node(name)
                    }
                }
                _ => DynaComponent::unknown_node(name),
            }
        }
    }
}

fn node_view_with_subs(node: &Node, spec: &mut WidgetSpec, _idx: usize, cx: &mut ViewContext<'_, DynaView>) -> DynaComponentView {
    let mut subs = Vec::new();
    let mut idx = 0;
    for stmt in node.body.stmts.iter() {
        println!("stmt: {:?}", stmt);
        match stmt {
            Stmt::Node(node) => {
                let compo = parse_node(&node.name.text, &node, spec, idx, cx);
                subs.push(compo);
                idx += 1;
            }
            Stmt::Expr(Expr::Call(call)) => {
                let node: Node = call.clone().into();
                println!("node: {:?}", node);
                let compo = parse_node(&node.name.text, &node, spec, idx, cx);
                subs.push(compo);
                idx += 1;
            }
            _ => (),
        }
    }
    DynaComponentView::Subs(subs)
}

fn add_row(div: Div, node: &Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, view: &DynaComponentView) -> Div {
    let mut row = row().gap_4();
    match view {
        DynaComponentView::Subs(subs) => {
            // TODO: calls should be parsed into nodes at an earlier stage
            for (i, n) in node.body.stmts.iter().enumerate() {
                match n {
                    Stmt::Node(node) => {
                        let compo = subs.get(i).unwrap();
                        let view = &compo.view;
                        let builder = &compo.builder;
                        row = builder(row, node, spec, cx, view);
                    }
                    Stmt::Expr(Expr::Call(call)) => {
                        let node: Node = call.clone().into();
                        let compo = subs.get(i).unwrap();
                        let view = &compo.view;
                        let builder = &compo.builder;
                        row = builder(row, &node, spec, cx, view);
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
    div.child(row)
}

// TODO: merge add_row and add_col
fn add_col(div: Div, node: &Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, view: &DynaComponentView) -> Div {
    let mut col = col().w_full().gap_1();
    match view {
        DynaComponentView::Subs(subs) => {
            // TODO: calls should be parsed into nodes at an earlier stage
            for (i, n) in node.body.stmts.iter().enumerate() {
                match n {
                    Stmt::Node(node) => {
                        let compo = subs.get(i).unwrap();
                        let view = &compo.view;
                        let builder = &compo.builder;
                        col = builder(col, node, spec, cx, view);
                    }
                    Stmt::Expr(Expr::Call(call)) => {
                        let node: Node = call.clone().into();
                        let compo = subs.get(i).unwrap();
                        let view = &compo.view;
                        let builder = &compo.builder;
                        col = builder(col, &node, spec, cx, view);
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
    div.child(col)
}

// TODO: currently only support onclick property
fn add_button(mut div: Div, node: &Node, _spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, _view: &DynaComponentView) -> Div {
    println!("node: {}", node);
    let text_arg = node.args.get(0);
    if let Some(Arg::Pos(Expr::Str(text))) = text_arg {
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
                        } else if text == "action" {
                            let v = pair.value.clone();
                            if let Expr::Str(action) = *v {
                                button = button.on_click_mut(cx, move |_this, _ev, cx| {
                                    if action == "collect" {
                                        GlobalDataStoreCollectAction::update_global(cx, |_ds, _cx| {});
                                    }
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

fn add_text(mut div: Div, node: &Node, spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>, _view: &DynaComponentView) -> Div {
    let text_arg = node.args.get(0);
    println!("text arg: {}", node.args);
    if let Some(arg) = text_arg {
        let val = spec.eval_expr(&arg.get_expr());
        div = div.child(format!("{}", val.repr()));
    }
    div
}

pub fn add_list(mut div: Div, node: &Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>, _view: &DynaComponentView) -> Div {
    let data = match node.args.get(0) {
        Some(Arg::Pos(expr)) => spec.eval_expr(&expr),
        _ => Value::Nil,
    };
    if let Value::Array(array) = data {
        let array = array.iter().map(|v| v.repr().into()).collect::<Vec<SharedString>>();
        div = div.child(cx.new_view(|cx| List::new(cx, array)));
        div
    } else {
        div
    }
}

pub fn add_table_view( node: &Node, spec: &mut WidgetSpec, idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {
    let len = node.args.len();
    let table_id = match node.args.get(0) {
        Some(Arg::Pos(Expr::Str(id))) => id,
        _ => format!("table_{}", idx),
    }; 
    if len == 2 {
        let grid = match node.args.get(1) {
            Some(Arg::Pos(expr)) => spec.eval_expr(&expr),
            _=> Value::Nil,
        };
        if let Value::Grid(grid) = grid {
            let view = cx.new_view(|cx| Table::from_grid(cx, table_id, grid));
            view.into()
        } else {
            cx.new_view(|cx| Table::from_grid(cx, table_id, Grid::default())).into()
        }
    } else if len == 3 {
        let config = match node.args.get(1) {
            Some(Arg::Pos(expr)) => spec.eval_expr(&expr),
            _=> Value::Nil,
        };
        let data = match node.args.get(2) {
            Some(Arg::Pos(expr)) => spec.eval_expr(&expr),
            _=> Value::Nil,
        };
        let head = convert_value_to_grid_head(&config);
        let config = convert_value_to_table_config(&config);
        let data = convert_value_to_table_data(&data, &config);
        cx.new_view(|cx| Table::new(cx, table_id, config, data, head)).into()
    } else {
        cx.new_view(|cx| Table::from_grid(cx, table_id, Grid::default())).into()
    }
}

pub fn add_input_view(node: &Node, spec: &mut WidgetSpec, _idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {
    let text = match node.args.get(0) {
        Some(Arg::Pos(expr)) => match expr {
            Expr::Str(text) => text,
            Expr::Ident(_) => spec.eval_expr(&expr).repr(),
            _ => "".to_string(),
        }
        _ => "".to_string(),
    };
    let view = cx.new_view(|cx| {
        let mut input = TextInput::new(cx);
        input.set_text(text, cx);
        input
    });
    view.into()
}

pub fn add_view_clone(div: Div, _node: &Node, _spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>, view: &DynaComponentView) -> Div {
    match view {
        DynaComponentView::View(view) => div.child(view.clone()),
        DynaComponentView::Subs(_) => div,
        DynaComponentView::None => div,
    }
}

fn node_to_dynaview(node: &Node, spec: &mut WidgetSpec, _idx: usize, compact: bool, cx: &mut ViewContext<'_, DynaView>) -> View<DynaView> {
    let tab_widget = WidgetSpec::from_ast_node(node, &spec.path, spec.scope.clone());
    let view = cx.new_view(|cx| {
        let mut view = DynaView::new(cx);
        view.set_spec(tab_widget);
        view.update_spec(cx);
        if compact {
            view.set_compact(true);
        }
        view
    });
    view
}

pub fn node_view_tabs( node: &Node, spec: &mut WidgetSpec, _idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {
    let mut tabs = Vec::new();
    let mut control = None;
    for (idx, stmt) in node.body.stmts.iter().enumerate() {
        match stmt {
            Stmt::Node(node) => {
                let tag = &node.name.text;
                if tag == "tab" {
                    let tab_view = node_to_dynaview(node, spec, idx, false, cx);
                    let name = match node.args.get(0) {
                        Some(Arg::Pos(Expr::Str(name))) => name,
                        _ => format!("view {}", idx),
                    };
                    let title = match node.args.get(1) {
                        Some(Arg::Pos(Expr::Str(title))) => title,
                        _ => name.clone(),
                    };
                    tabs.push((name, title, tab_view));
                } else if tag == "control" {
                    let control_view = node_to_dynaview(node, spec, idx, true, cx);
                    control = Some(control_view);
                }
            }
            _ => (),
        }
    }
    let len = tabs.len();
    cx.new_view(|cx| {
        let mut tabpane = TabPane::new(cx);
        for tab in tabs.into_iter() {
            tabpane = tabpane.add(cx.new_view(|cx| TabView::new(cx, tab.0, tab.1, tab.2)));
        }
        if len > 0 {
            tabpane.set_active(0, cx);
        }
        if let Some(control) = control {
            tabpane = tabpane.control(control.into());
        }
        tabpane
    })
    .into()
}



pub fn node_view_dropzone(node: &Node, _spec: &mut WidgetSpec, _idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {

    let mut ondrop: Option<Box<dyn Fn(&str, &mut ViewContext<DropZone>)>> = None;
        for stmt in node.body.stmts.iter() {
            if let Stmt::Expr(Expr::Pair(pair)) = stmt {
                match &pair.key {
                    Key::NamedKey(Name { text }) => {
                        if text == "ondrop" {
                            let expr = &pair.value;
                            match expr.as_ref() {
                                Expr::Lambda(lambda) => {
                                    let lb = lambda.clone();
                                    ondrop = Some(Box::new(move |f, cx| {
                                        // .run_lambda(&lambda);
                                        println!("ondrop: {:?}", f);
                                        // let cl = lambda.clone();
                                        // GlobalSpecState::global(cx).run_lambda(&cl);
                                        println!("ondrop: {:?}", lb);
                                        GlobalSpecState::update_global(cx, |state, _cx| {
                                            state.scope().borrow_mut().set_local_val("f", Value::Str(f.to_string()));
                                            state.run_lambda(&lb);
                                        });
                                        cx.notify();
                                    }));
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    cx.new_view(|cx| {
        let mut drop = DropZone::new(cx);
        if let Some(ondrop) = ondrop {
            drop = drop.on_drop(ondrop);
        }
        drop
    })
    .into()
}

pub fn node_view_dropdown(node: &Node, spec: &mut WidgetSpec, _idx: usize, cx: &mut ViewContext<'_, DynaView>) -> AnyView {
    // get options from props
    let title = match node.args.get(0) {
        Some(Arg::Pos(Expr::Str(s))) => s.clone(),
        _ => "dropdown".into(),
    };
    println!("title: {:?}", title);
    println!("args[1]: {:?}", node.args.get(1));
    let options = match node.args.get(1) {
        Some(arg) => {
            let val = spec.eval_expr(&arg.get_expr());
            if let Value::Array(array) = val {
                array
            } else {
                vec![]
            }
        }
        _ => vec![],
    };
    println!("options: {:?}", options);
    let options = options.iter().map(|o| o.to_string().into()).collect::<Vec<SharedString>>();
    let view = cx.new_view(|cx| Dropdown::new(ElementId::Name(title.into()), options, None, cx));
    view.into()
}

pub fn convert_value_to_grid_head(value: &Value) -> Vec<(ValueKey, Value)> {    
    match value {
        Value::Array(array) => {
            let mut head = vec![];
            for item in array.iter() {
                match item {
                    Value::Obj(obj) => {
                        head.push((ValueKey::Str(obj.get_str_or("id", "")), item.clone()))
                    }
                    _ => (),
                }
            }
            head
        }
        _ => vec![],
    }
}

pub fn convert_value_to_table_config(value: &Value) -> Vec<ColConfig> {
    match value {
        Value::Array(array) => {
            let mut cols = vec![];
            for (idx, item) in array.iter().enumerate() {
                match item {
                    Value::Obj(obj) => {
                        let col = ColConfig {
                            idx,
                            id: obj.get_str_or("id", "").into(),
                            title: obj.get_str_or("name", ""),
                            width: obj.get_or("width", Value::Float(0.0)).into(),
                            format: obj.get_or("format", Value::from("Text")).into(),
                            options: obj.get_array_of("options").iter().map(|s| s.repr()).collect::<Vec<String>>(),
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

pub fn convert_value_to_table_data(value: &Value, config: &Vec<ColConfig>) -> Vec<Vec<Value>> {
    match value {
        Value::Array(array) => {
            let mut rows = Vec::new();
            for item in array.iter() {
                let mut cells = Vec::new();
                for col in config.iter() {
                    match item {
                        Value::Obj(obj) => {
                            let cell = obj.get_or(&col.id, Value::Nil);
                            cells.push(cell.clone());
                        }
                        _ => (),
                    }
                }
                rows.push(cells);
            }
            rows
        }
        _ => vec![],
    }
}

impl Render for DynaView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = if self.compact { row().gap_4() } else { col().size_full() };

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
