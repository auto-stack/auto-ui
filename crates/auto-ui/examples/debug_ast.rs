#[cfg(feature = "transpiler")]
use auto_lang::Parser;

#[cfg(feature = "transpiler")]
fn main() {
    let source = r#"
enum Msg { Inc, Dec }

fn main() {
    let x = Msg.Inc
    is x {
        Msg.Inc => print("inc")
        Msg.Dec => print("dec")
    }
}
"#;

    let mut parser = Parser::from(source);

    match parser.parse() {
        Ok(ast) => {
            println!("=== Parse Successful ===");
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
}
