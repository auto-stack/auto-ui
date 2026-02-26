//! Test example for Vue backend transpilation

#[cfg(feature = "vue-backend")]
fn main() {
    use auto_lang::Parser;
    use auto_ui::trans::api::vue::transpile_vue_ast;

    let source = r#"
type Counter {
    count int

    fn view() View {
        col {
            button "+" { onclick: CounterMsg.Inc }
            h2 "Count"
            button "-" { onclick: CounterMsg.Dec }
        }
    }
}
"#;

    let mut parser = Parser::from(source);

    match parser.parse() {
        Ok(ast) => {
            println!("=== Parse Successful ===");
            match transpile_vue_ast(&ast) {
                Ok(vue_code) => {
                    println!("=== Generated Vue SFC ===");
                    println!("{}", vue_code);
                }
                Err(e) => {
                    eprintln!("Vue transpilation failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse failed: {:?}", e);
        }
    }
}

#[cfg(not(feature = "vue-backend"))]
fn main() {
    println!("This example requires the 'vue-backend' feature.");
    println!("Run with: cargo run -p auto-ui --features vue-backend --example test_counter_vue");
}
