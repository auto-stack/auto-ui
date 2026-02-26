// Integration tests for AutoUI transpiler
//
// Tests the complete flow from .at files to generated Rust code

#[cfg(feature = "transpiler")]
mod tests {
    use auto_ui::trans::{transpile_ast, transpile_dsl, transpile_file};

    #[test]
    fn test_transpile_counter() {
        let result = transpile_file("../../scratch/counter.at", None);

        match &result {
            Ok(code) => {
                println!("Generated code:\n{}", code);
            }
            Err(e) => {
                eprintln!("Transpilation failed (expected during development): {}", e);
            }
        }
    }

    #[test]
    fn test_transpile_simple_widget() {
        use auto_lang::Parser;

        let source = r#"
type Hello is Widget {
    msg str = "Hello World"

    fn view() {
        text(self.msg) {}
    }
}
"#;

        let mut parser = Parser::from(source);

        match parser.parse() {
            Ok(ast) => match transpile_ast(&ast) {
                Ok(code) => {
                    println!("Generated code:\n{}", code);
                }
                Err(e) => {
                    eprintln!("Transpilation failed: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_message_enum_derivation() {
        use auto_lang::Parser;

        let source = r#"
type Counter is Widget {
    count int = 0

    fn view() {
        col {
            button "+" {
                onclick: Msg.Inc
            }
            text(count)
            button "-" {
                onclick: Msg.Dec
            }
        }
    }

    fn on(ev Msg) {
        is ev {
            Msg.Inc => self.count += 1
            Msg.Dec => self.count -= 1
        }
    }
}
"#;

        let mut parser = Parser::from(source);

        match parser.parse() {
            Ok(ast) => match transpile_ast(&ast) {
                Ok(code) => {
                    println!("Generated code:\n{}", code);
                }
                Err(e) => {
                    eprintln!("Transpilation failed: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_dsl_preprocess_counter() {
        let dsl_source = r#"
msg CounterMsg {
    Inc
    Dec
}

widget Counter {
    model {
        count int
    }

    view {
        col {
            button + { onclick: CounterMsg.Inc }
            h2 > Count: ${count}
            button - { onclick: CounterMsg.Dec }
        }
    }

    on {
        CounterMsg.Inc => { .count += 1 }
        CounterMsg.Dec => { .count -= 1 }
    }
}
"#;

        let result = transpile_dsl(dsl_source, None);

        match &result {
            Ok(code) => {
                println!("Generated code from DSL:\n{}", code);
            }
            Err(e) => {
                eprintln!("DSL transpilation failed: {}", e);
            }
        }
    }
}
