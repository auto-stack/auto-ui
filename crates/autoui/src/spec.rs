use autolang::eval::Evaler;
use autolang::interpret;
use autoval::*;
use autolang::ast;
use autolang::scope::{Universe, Meta};
use autolang::ast::{Expr, Fn, Args, Arg};
use autolang::interp::Interpreter;
use crate::dyna::state::State;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::cell::RefMut;
use std::default::Default;

pub struct Spec {
    path: String,
    source: String,
    interpreter: Rc<RefCell<Interpreter>>,
    scope: Rc<RefCell<Universe>>,
}

impl Default for Spec {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Interpreter::new())))
    }
}

impl Spec {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        let scope = interpreter.as_ref().borrow().scope.clone();
        Self { path: String::new(), source: String::new(), interpreter, scope }
    }

    pub fn from_file(path: &str, interpreter: Rc<RefCell<Interpreter>>) -> Self {
        let mut spec = Self::new(interpreter);
        spec.read_file(path);
        spec
    }

    pub fn read_str(&mut self, source: &str) {
        match self.interpreter.borrow_mut().interpret(source) {
            Ok(_) => {
                self.source = source.to_string();
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

    pub fn scope_shared(&self) -> Rc<RefCell<Universe>> {
        self.interpreter.as_ref().borrow().scope.clone()
    }

    fn scope(&self) -> Ref<Universe> {
        self.scope.as_ref().borrow()
    }

    fn scope_mut(&mut self) -> RefMut<Universe> {
        self.scope.as_ref().borrow_mut()
    }

    pub fn set_state(&self, state: &mut State) {
        let widget = &self.scope().widget();
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
        match &self.scope().widget() {
            Value::Widget(widget) => {
                let metaid = &widget.view_id;
                match metaid {
                    MetaID::View(metaid) => {
                        let meta = self.scope().lookup_meta(metaid);
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

    pub fn get_app_node(&self) -> Option<Node> {
        let result = self.interpreter.as_ref().borrow().result.clone();
        match result {
            Value::Node(node) => Some(node),
            _ => None,
        }
    }

    pub fn get_widget(&self) -> Value {
        self.scope().widget().clone()
    }

    pub fn eval_value(&mut self, value: &Value) -> Value {
        match value {
            Value::Str(s) => self.interpreter.as_ref().borrow_mut().evaler.eval_expr(&Expr::Str(s.clone())),
            _ => value.clone(),
        }
    }

    pub fn eval_ident(&mut self, ident: &Expr) -> Value {
        self.interpreter.as_ref().borrow_mut().evaler.eval_expr(ident)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        self.interpreter.as_ref().borrow_mut().evaler.eval_expr(expr)
    }

    pub fn run_lambda(&mut self, lambda: &Fn) -> Value {
        self.interpreter.as_ref().borrow_mut().evaler.eval_fn_call(lambda, &Args::new());
        let count = self.scope().lookup_val("count").unwrap_or(Value::Nil);
        println!("new count: {}", count);
        count
    }
}

#[derive(Clone)]
pub struct WidgetSpec {
    pub widget: Value,
    pub path: String,
    pub id: String,
    pub scope: Rc<RefCell<Universe>>,
}

impl Default for WidgetSpec {
    fn default() -> Self {
        Self::new(Value::Nil, ".", "", Rc::new(RefCell::new(Universe::default())))
    }
}

impl WidgetSpec {
    pub fn new(widget: Value, path: &str, id: &str, scope: Rc<RefCell<Universe>>) -> Self {
        Self { widget, path: path.to_string(), id: id.to_string(), scope }
    }

    pub fn from_ast_node(node: &ast::Node, path: &str, scope: Rc<RefCell<Universe>>) -> Self {
        // make node into a `View` meta and put it in the scope
        // TODO: this id may not be unique
        let node_name = node.name.text.clone();
        let node_arg0 = match node.args.get(0) {
            Some(Arg::Pos(Expr::Str(s))) => s.clone(),
            _ => "_".to_string(),
        };
        let body_id = format!("{}_{}.body", node_name, node_arg0);
        scope.borrow_mut().define(&body_id, Rc::new(Meta::Body(node.body.clone())));
        let widget = Widget { name: node_name.clone(), model: Model::new(), view_id: MetaID::Body(body_id.clone()) };
        Self{ widget: Value::Widget(widget), path: path.to_string(), id: body_id, scope }
    }

    pub fn read_str(&mut self, source: &str) {
        match interpret(source) {
            Ok(result) => {
                self.scope = result.scope.clone();
                let scope = result.scope.borrow();
                let widget = scope.widget().clone();
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
        let mut spec = Self::new(Value::Nil, path, path, Rc::new(RefCell::new(Universe::new())));
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
        let uni = self.scope.clone();
        let mut evaler = Evaler::new(uni);
        match value {
            Value::Str(s) => evaler.eval_expr(&Expr::Str(s.clone())),
            _ => value.clone(),
        }
    }

    pub fn eval_ident(&mut self, ident: &Expr) -> Value {
        let uni = self.scope.clone();
        let mut evaler = Evaler::new(uni);
        evaler.eval_expr(ident)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        let uni = self.scope.clone();
        let mut evaler = Evaler::new(uni);
        evaler.eval_expr(expr)
    }

    pub fn run_lambda(&mut self, lambda: &Fn) -> Value {
        let uni = self.scope.clone();
        let mut evaler = Evaler::new(uni);
        evaler.eval_fn_call(lambda, &Args::new())
    }

    pub fn eval_stmt(&mut self, stmt: &ast::Stmt) -> Value {
        let uni = self.scope.clone();
        let mut evaler = Evaler::new(uni);
        return evaler.eval_stmt(stmt)
    }
}
