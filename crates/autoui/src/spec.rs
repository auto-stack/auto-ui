use autolang::ast::Code;
use autolang::parse;

pub struct Spec {
    code: Code,
    source: String,
}

impl Spec {
    pub fn new() -> Self {
        Self { code: Code::default(), source: String::new() }
    }


    pub fn read_str(source: &str) -> Spec {
        match parse(source) {
            Ok(code) => {
                println!("{:?}", code);
                Self { code, source: source.to_string() }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
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
                button("-") {
                    onclick: || count = count - 1
                }
            }
        }
        "#;
        let spec = Spec::read_str(source);
        println!("{:?}", spec.source);
        println!("{:?}", spec.code);
    }
}
