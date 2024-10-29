use autolang::ast::{Code, Expr, Stmt, View, Widget};
use autolang::parse;
use crate::dyna::state::State;

pub struct Spec {
    code: Code,
    source: String,
}

impl Spec {
    pub fn new() -> Self {
        Self { code: Code::default(), source: String::new() }
    }

    pub fn read_str(&mut self, source: &str) {
        let code = parse(source);
        match code {
            Ok(code) => {
                self.code = code;
                self.source = source.to_string();
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn read_file(&mut self, path: &str) {
        let source = std::fs::read_to_string(path).unwrap();
        println!("source :{}", source);
        self.read_str(&source);
    }

    fn find_widget(&self) -> &Widget {
        for stmt in &self.code.stmts {
            match stmt {
                Stmt::Widget(widget) => return widget,
                _ => {}
            }
        }
        panic!("expected widget statement");
    }

    pub fn set_state(&self, state: &mut State) {
        let widget = self.find_widget();
        let model = &widget.model;
        for var in &model.vars {
            let name = var.name.text.clone();
            match &var.expr {
                Expr::Int(n) => {
                    state.set_int(&name, *n);
                }
                Expr::Str(s) => {
                    state.set_str(&name, s.clone());
                }
                Expr::Bool(b) => {
                    state.set_bool(&name, *b);
                }
                _ => panic!("expected int or str value"),
            }
        }
    }

    pub fn get_view(&self) -> &View{
        &self.find_widget().view
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_str() {
        let source = r#"
        widget counter {
            model {
                var count = 0
            }
            view {
                button("+") {
                    onclick: || count = count + 1
                }
                text(count)
            }
        }
        "#;
        let mut spec = Spec::new();
        spec.read_str(source);
        // let view = DynaView::new(&mut ViewContext::new(cx));
    }
}
