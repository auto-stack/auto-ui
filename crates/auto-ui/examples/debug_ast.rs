#[cfg(feature = "transpiler")]
use auto_lang::Parser;
#[cfg(feature = "transpiler")]
use std::rc::Rc;
#[cfg(feature = "transpiler")]
use std::cell::RefCell;
#[cfg(feature = "transpiler")]
use auto_lang::Universe;

#[cfg(feature = "transpiler")]
fn main() {
    let source = r#"
enum Msg {
    Inc
    Dec
}

type Counter {
    count int = 0

    fn view() {
        col {
            button "+" {
                onclick: Msg.Inc
            }
            text(count)
        }
    }
}
"#;

    let universe = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(source, universe);

    match parser.parse() {
        Ok(ast) => {
            println!("=== Parse Successful ===");
            println!("AST: {:#?}", ast);
            println!();
            println!("=== Statements ===");
            for (i, stmt) in ast.stmts.iter().enumerate() {
                println!("Stmt {}:", i);
                println!("  {:#?}", stmt);
                println!();
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
}
