use autogui::widget;
use autolang::eval::Evaler;
use autolang::interpret;
use autoval::value::*;
use autoval::value;
use autolang::ast;
use autolang::scope::{Universe, Meta};
use autolang::ast::{Expr, Lambda, Fn, Args};
use crate::dyna::state::State;
use autogui::widget::button::Button;
use gpui::*;

pub struct Spec {
    path: String,
    source: String,
    scope: Option<Universe>,
}

impl Spec {
    pub fn new() -> Self {
        Self { path: String::new(), source: String::new(), scope: None }
    }

    pub fn read_str(&mut self, source: &str) {
        match interpret(source) {
            Ok(result) => {
                self.path = "".to_string();
                self.source = source.to_string();
                self.scope = Some(result.scope);
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn read_file(&mut self, path: &str) {
        self.path = path.to_string();
        let source = std::fs::read_to_string(path).unwrap();
        println!("source :{}", source);
        self.read_str(&source);
    }

    pub fn reload(&mut self) {
        if !self.path.is_empty() {
            let path = self.path.clone();
            self.read_file(&path);
        }
    }

    pub fn set_state(&self, state: &mut State) {
        let widget = &self.scope.as_ref().unwrap().widget;
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
        match &self.scope.as_ref().unwrap().widget {
            Value::Widget(widget) => {
                let metaid = &widget.view;
                match metaid {
                    MetaID::View(metaid) => {
                        let meta = self.scope.as_ref().unwrap().get_symbol(metaid);
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

    pub fn eval_value(&mut self, value: &Value) -> Value {
        let mut evaler = Evaler::new(self.scope.as_mut().unwrap());
        match value {
            Value::Str(s) => evaler.eval_expr(&Expr::Str(s.clone())),
            _ => value.clone(),
        }
    }

    pub fn eval_ident(&mut self, ident: &Expr) -> Value {
        let mut evaler = Evaler::new(self.scope.as_mut().unwrap());
        evaler.eval_expr(ident)
    }

    pub fn run_lambda(&mut self, lambda: Lambda) -> Value {
        let mut evaler = Evaler::new(self.scope.as_mut().unwrap());
        let fn_decl: &Fn = &lambda.into();
        evaler.eval_fn_call_no_enter(fn_decl, &Args::new());
        self.scope.as_mut().map(|s| {
            let cnt = s.get_local("count");
            println!("cnt: {:?}", cnt);
            cnt
        }).unwrap_or(Value::Nil)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_read_str() {
    //     let source = r#"
    //     widget counter {
    //         model {
    //             var count = 0
    //         }
    //         view {
    //             button("+") {
    //                 onclick: || count = count + 1
    //             }
    //             text(count)
    //         }
    //     }
    //     "#;
    //     // let mut spec = Spec::new();
    //     // spec.read_str(source);
    //     // let view = DynaView::new(&mut ViewContext::new(cx));
    // }
}
