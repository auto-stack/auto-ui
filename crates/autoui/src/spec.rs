use autolang::eval::Evaler;
use autolang::interpret;
use autoval::value::*;
use autolang::ast;
use autolang::scope::{Universe, Meta};
use autolang::ast::{Expr, Fn, Args};
use crate::dyna::state::State;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Spec {
    path: String,
    source: String,
    pub scope: Rc<RefCell<Universe>>,
    pub result: Value,
}

impl Spec {
    pub fn new() -> Self {
        Self { path: String::new(), source: String::new(), scope: Rc::new(RefCell::new(Universe::new())), result: Value::Nil }
    }

    pub fn read_str(&mut self, source: &str) {
        match interpret(source) {
            Ok(result) => {
                self.source = source.to_string();
                self.scope = Rc::new(RefCell::new(result.scope));
                self.result = result.result;
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn read_file(&mut self, path: &str) {
        self.path = path.to_string();
        let source = std::fs::read_to_string(path).unwrap();
        self.read_str(&source);
    }

    pub fn reload(&mut self) {
        println!("re read file: {}", self.path);
        if !self.path.is_empty() {
            let path = self.path.clone();
            self.read_file(&path);
        }
    }

    pub fn set_state(&self, state: &mut State) {
        let widget = &self.scope.as_ref().borrow().widget;
        match widget {
            Value::Widget(widget) => {
                let model = &widget.model;
                for (name, expr) in &model.values {
                    let name = match name {
                        ValueKey::Str(name) => name,
                        _ => panic!("expected var name"),
                    };

                    match expr {
                        Value::Int(n) => {
                            state.set_int(&name, *n);
                        }
                        Value::Str(s) => {
                            state.set_str(&name, s.clone());
                        }
                        Value::Bool(b) => {
                            state.set_bool(&name, *b);
                        }
                        _ => panic!("expected int or str value"),
                    }
                }
            }
            _ => panic!("expected widget"),
        }
    }

    pub fn get_view(&self) -> ast::View {
        match &self.scope.as_ref().borrow().widget {
            Value::Widget(widget) => {
                let metaid = &widget.view_id;
                match metaid {
                    MetaID::View(metaid) => {
                        let meta = self.scope.as_ref().borrow().get_symbol(metaid);
                        match meta {
                            Some(meta) => {
                                match meta.as_ref() {
                                    Meta::View(view) => view.clone(),
                                    _ => panic!("expected view"),
                                }
                            }
                            None => panic!("expected view"),
                        }
                    }
                    _ => panic!("expected view"),
                }
            }
            _ => panic!("expected widget"),
        }
    }

    pub fn get_app(&self) -> Option<&Node> {
        // self.scope.as_ref().unwrap().get_node("app")
        None
    }

    pub fn get_widget(&self) -> Value {
        self.scope.as_ref().borrow().widget.clone()
    }

    pub fn eval_value(&mut self, value: &Value) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        match value {
            Value::Str(s) => evaler.eval_expr(&Expr::Str(s.clone())),
            _ => value.clone(),
        }
    }

    pub fn eval_ident(&mut self, ident: &Expr) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_expr(ident)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_expr(expr)
    }

    pub fn run_lambda(&mut self, lambda: &Fn) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_fn_call(lambda, &Args::new());
        let count = uni.lookup_val("count").unwrap_or(Value::Nil);
        println!("new count: {}", count);
        count
    }
}

pub struct WidgetSpec {
    pub widget: Value,
    pub path: String,
    scope: Rc<RefCell<Universe>>,
}

impl WidgetSpec {
    pub fn new(widget: Value, path: &str, scope: Rc<RefCell<Universe>>) -> Self {
        Self { widget, path: path.to_string(), scope }
    }

    pub fn read_str(&mut self, source: &str) {
        match interpret(source) {
            Ok(result) => {
                let scope = result.scope;
                let widget = scope.widget.clone();
                self.scope = Rc::new(RefCell::new(scope));
                self.widget = widget;
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn read_file(&mut self, path: &str) {
        self.path = path.to_string();
        let source = std::fs::read_to_string(path).unwrap();
        self.read_str(&source);
    }

    pub fn from_file(path: &str) -> Self {
        let mut spec = Self::new(Value::Nil, path, Rc::new(RefCell::new(Universe::new())));
        spec.read_file(path);
        spec
    }

    pub fn reload(&mut self) {
        self.read_file(self.path.clone().as_str());
    }

    pub fn get_ast_view(&self) -> Option<ast::View> {
        match &self.widget {
            Value::Widget(widget) => {
                self.scope.as_ref().borrow().lookup_view(&widget.view_id)
            }
            _ => None,
        }
    }

    pub fn eval_value(&mut self, value: &Value) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        match value {
            Value::Str(s) => evaler.eval_expr(&Expr::Str(s.clone())),
            _ => value.clone(),
        }
    }

    pub fn eval_ident(&mut self, ident: &Expr) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_expr(ident)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_expr(expr)
    }

    pub fn run_lambda(&mut self, lambda: &Fn) -> Value {
        let mut uni = self.scope.as_ref().borrow_mut();
        let mut evaler = Evaler::new(&mut uni);
        evaler.eval_fn_call(lambda, &Args::new());
        uni.lookup_val("count").unwrap_or(Value::Nil)
    }
}
