use auto_gen::*;
use auto_val::{Obj, AutoPath, AutoStr, Value};
use auto_atom::Atom;
use auto_lang::config::AutoConfig;
use auto_lang::parser::Parser;
use auto_lang::scope::Meta;
use auto_ui::widget::WidgetInfo;
use auto_lang::ast::Type;
use auto_lang::AutoResult;
use auto_lang::ast::{Node, Name};
use auto_lang::eval::{Evaler, EvalMode};
use auto_lang::universe::Universe;
use std::rc::Rc;
use std::cell::RefCell;
fn gen_from_data() {
    let mut data = AutoConfig::from_file(AutoPath::new("crates/auto-ui/examples/hello_data.at").path(), &Obj::new()).unwrap();
    let atom = Atom::node(data.root);
    println!("{}", atom);

    // 3. feed atom to generator and generate code
    let gen = AutoGen::new()
        .molds(vec![Mold::from_file(AutoPath::new("assets/templates/story.at.rs"))])
        .data(atom)
        .out(AutoPath::new("crates/auto-ui/examples/"));
    let result = gen.gen();
    println!("{}", result);
}

fn gen_from_auto() -> AutoResult<()> {
    // 1. read auto code
    let code = AutoStr::from(std::fs::read_to_string("crates/auto-ui/examples/hello.at").unwrap());
    // 2. Use auto-lang parser to parse the code
    let universe = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(&code, universe.clone());
    let ast = parser.parse()?;
    // view type
    let view_type = universe.borrow().lookup_type("Hello");
    let mut root = auto_val::Node::new("hello");
    root.set_prop("name", "hello");
    if let Some(view_type) = view_type {
        if let Meta::Type(Type::User(user_decl)) = &*view_type {
            println!("{}", user_decl);
            let widget_info = WidgetInfo::from(user_decl);
            // convert widget info to a story node
            let story_node = widget_info.to_node("story");
            root.add_kid(story_node);
        }
    }
    // app
    let app = ast.stmts.iter().for_each(|stmt| {
        if let auto_lang::ast::Stmt::Fn(fn_decl) = stmt {
            if fn_decl.name.text == "main" {
                let body = &fn_decl.body;
                let last_stmt = body.stmts.last().unwrap();
                if let auto_lang::ast::Stmt::Node(node) = last_stmt {
                    // convert app code to rust code
                    if node.name.text == "app" {
                        let title = node.args.args[0].repr();
                        let mut app_node = auto_val::Node::new("app");
                        app_node.set_prop("title", title);
                        root.add_kid(app_node);
                    }
                }
            }
        }
    });

    println!("{}", Value::Node(root.clone()).pretty(3));

    let atom = Atom::node(root);
    println!("{}", atom);

    // 3. feed atom to generator and generate code
    let gen = AutoGen::new()
        .molds(vec![Mold::from_file(AutoPath::new("assets/templates/story.at.rs"))])
        .data(atom)
        .out(AutoPath::new("crates/auto-ui/examples/"));
    let result = gen.gen();
    println!("{}", result);

    Ok(())
}

fn main() {
    gen_from_auto().unwrap();
    // gen_from_data();
}
