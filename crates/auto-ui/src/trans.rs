use auto_lang::trans::Transpiler;
use auto_lang::ast::{Code, Stmt, Fn, TypeDecl};
use auto_val::Node;
use std::io::Write;
use auto_lang::AutoResult;

pub struct UITranspiler {

}

impl Transpiler for UITranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()> {
        for stmt in ast.stmts.into_iter() {
            match stmt {
                Stmt::Fn(fn_stmt) => {
                    if fn_stmt.name.text == "main" {
                        self.do_main(&fn_stmt)?;
                    } else {
                        self.do_fn(&fn_stmt)?;
                    }
                }
                Stmt::TypeDecl(type_decl) => {
                    self.do_type(&type_decl)?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl UITranspiler {
    fn do_main(&mut self, fn_stmt: &Fn) -> AutoResult<()> {
        let mut main = Node::new("main");
        Ok(())
    }

    fn do_fn(&mut self, fn_stmt: &Fn) -> AutoResult<()> {
        println!("do_fn");
        Ok(())
    }

    fn do_type(&mut self, type_decl: &TypeDecl) -> AutoResult<()> {
        println!("do_type");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang::parse;

    #[test]
    fn test_ui_transpiler() {
        let code = r#"
type Hello {
    msg str

    fn view() {
        center {
            label(self.msg) {}
        }
    }
}

fn main() {
    app("Hello Example") {
        hello("Hello World!") {}
    }
}
        "#;
        let mut transpiler = UITranspiler {};
        let mut out = Vec::new();
        let ast = parse(code).unwrap();
        let result = transpiler.transpile(ast, &mut out);
        println!("{}", String::from_utf8(out).unwrap());
    }
}