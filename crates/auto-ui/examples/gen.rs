use auto_lang::parse_with_scope;
use auto_val::shared;
use auto_lang::Universe;
use auto_ui::trans::GpuiTrans;
use auto_lang::trans::Trans;
use auto_ui::trans::Templates;
use auto_lang::scope::Meta;
use auto_lang::ast::Name;
use std::rc::Rc;
use auto_lang::token::TokenKind;
use auto_lang::parser::BlockParser;
use auto_lang::parser::Parser;
use auto_lang::ast::Body;
use auto_lang::parser::ParseError;
use auto_lang::ast::Stmt;
use auto_lang::ast::Expr;

pub struct MarkdownParser;

impl BlockParser for MarkdownParser {
    fn parse(&self, parser: &mut Parser) -> Result<Body, ParseError> {
        let mut code = Vec::new();
        while !parser.is_kind(TokenKind::RBrace) {
            code.push(parser.cur.text.clone());
            parser.next();
        }
        let mut body = Body::new();
        body.stmts.push(Stmt::Expr(Expr::Str(code.join("\n").into())));
        Ok(body)
    }
}

fn gen_example(example: &str) {
    let code = std::fs::read_to_string(format!("crates/auto-ui/examples/{}.at", example)).unwrap();
    let universe = shared(Universe::new());
    // TODO: import real theme to scope
    universe.borrow_mut().define("theme", Rc::new(Meta::Ref("theme".into())));
    let mut trans = GpuiTrans::new(example, universe.clone());
    let mut out = Vec::new();
    let mut parser = auto_lang::parser::Parser::new(code.as_str(), universe.clone());
    parser.add_special_block("markdown".into(), Box::new(MarkdownParser));
    let ast = parser.parse().unwrap();
    trans.trans(ast, &mut out).unwrap();
    println!("{}", String::from_utf8(out).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown() {
        let code = r#"
        markdown
        {
            # Hello
        }
        "#;
        let universe = shared(Universe::new());
        let mut parser = auto_lang::parser::Parser::new(code, universe.clone());
        parser.add_special_block("markdown".into(), Box::new(MarkdownParser));
        let ast = parser.parse().unwrap();
        println!("{:?}", ast);
    }
}

fn main() {
    let examples = vec![
    //     "hello",
    //     "login",
    //     "docks",
    //     "mark",
    //     "counter",
        "table",
    ];

    let story_template = Templates::story().unwrap();
    println!("{}", story_template);

    for example in examples {
        println!("Generating Example: {}", example);
        gen_example(example);
    }
}
