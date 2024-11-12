use std::sync::Arc;

use crate::spec::WidgetSpec;
use autogui::app::Viewable;
use autogui::widget::button::Button;
use autogui::widget::table::{Align, ColConfig, Row, Table};
use autolang::ast::{Expr, Key, Name};
use autoval::value::Value;
use crate::dyna::app::GlobalSpecState;
use gpui::*;

pub struct DynaView {
    spec: Option<WidgetSpec>,
    // state: State,
    builder: Option<Box<dyn Fn(Div, &mut WidgetSpec, &mut ViewContext<Self>) -> Div + 'static>>,
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
            v.update_spec();
            println!("update spec for widget");
        }).detach();
        Self {
            spec: None,
            // state,
            builder: None,
        }
    }
}

impl DynaView {
    pub fn reload(&mut self) {
        if let Some(spec) = self.spec.as_mut() {
            spec.reload();
        }
        self.update_spec();
    }

    pub fn set_spec(&mut self, spec: WidgetSpec) {
        self.spec = Some(spec);
    }

    pub fn from_file(&mut self, path: &str) {
        let spec = WidgetSpec::from_file(path);
        self.set_spec(spec);
    }

    pub fn update_spec(&mut self) {
        // self.spec.set_state(&mut self.state);

        self.builder = Some(Box::new(|div, spec, cx| {
            let mut div = div;

            let spec_view = spec.get_ast_view();
            if let Some(view) = spec_view {
                for (name, node) in view.nodes.iter() {
                    let node = node.clone();
                    div = parse_node(div, &name.text, &node, spec, cx);
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

fn parse_node(mut div: Div, name: &str, node: &autolang::ast::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    match name {
        "button" => div = add_button(div, &node, spec, cx),
        "text" => div = add_text(div, &node, spec, cx),
        "table" => div = add_table(div, &node, spec, cx),
        _ => {
            // try lookup widget in scope
            let widget = &spec.scope.borrow().widget;
            match widget {
                Value::Widget(w) => {
                    if w.name == name {
                        // make a new dynamic widget
                        let new_spec = WidgetSpec::new(widget.clone(), ".", "", spec.scope.clone());
                        div = add_widget(div, new_spec, cx);
                        cx.notify();
                    }
                }
                _ => ()
            }
        }
    };
    div
}


fn add_widget(div: Div, spec: WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    let mut view = DynaView::new(cx);
    view.set_spec(spec);
    view.update_spec();
    div.child(cx.new_view(|_cx| view))
}

fn add_button( mut div: Div, node: &autolang::ast::Node, _spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    let text_arg = node.args.get(0);
    if let Some(Expr::Str(text)) = text_arg {
        let mut button = Button::primary(text.as_str());

        // let onclick = node
            // .body
            // .get(&Key::NamedKey(Name::new("onclick".to_string())));
        // match onclick {
            // Some(Expr::Lambda(lambda)) => {
                // let lambda = lambda.clone();
                // button = button.on_click_mut(cx, move |this, _ev, cx| {
                    // this.spec.as_mut().unwrap().run_lambda(lambda.clone());
                    // cx.notify();
                // });
            // }
            // _ => (),
        // }
        div = div.child(button);
    }
    div
}

fn add_text( mut div: Div, node: &autolang::ast::Node, spec: &mut WidgetSpec, _cx: &mut ViewContext<'_, DynaView>,) -> Div {
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

pub fn add_table( mut div: Div, node: &autolang::ast::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>,) -> Div {
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
        let div = div()
            .flex()
            .flex_col()
            // .child(
            //     Button::primary("Refresh").on_click_mut(cx, |this, _ev, cx| {
            //         println!("reload");
            //         this.reload();
            //         cx.notify();
            //     }),
            // )
            .gap_2();

        if let Some(builder) = self.builder.as_ref() {
            builder(div, &mut self.spec.as_mut().unwrap(), cx)
        } else {
            div
        }
    }
}


fn parse_value_node(mut div: Div, node: &autoval::value::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    match node.name.as_str() {
        "button" => div = add_button_value(div, &node, spec, cx),
        "text" => div = add_text_value(div, &node, spec, cx),
        "table" => div = add_table_value(div, &node, spec, cx),
        _ => (),
    };
    div
}

fn add_button_value(mut div: Div, node: &autoval::value::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    let text_arg = node.args.array.get(0);
    if let Some(Value::Str(text)) = text_arg {
        let mut button = Button::primary(text.as_str());

        let onclick = node.get_prop("onclick");
        match onclick {
            Value::Lambda(id) => {
                button = button.on_click_mut(cx, move |this, _ev, cx| {
                    // this.spec.as_mut().unwrap().run_lambda(lambda.clone());
                    cx.notify();
                });
            }
            _ => (),
        }
        div = div.child(button);
    }
    div

}

fn add_text_value(mut div: Div, node: &autoval::value::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    div
}

fn add_table_value(mut div: Div, node: &autoval::value::Node, spec: &mut WidgetSpec, cx: &mut ViewContext<'_, DynaView>) -> Div {
    div
}
