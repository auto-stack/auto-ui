use auto_lang::parse_with_scope;
use auto_val::shared;
use auto_lang::Universe;
use auto_ui::trans::GpuiTrans;
use auto_lang::trans::Trans;
use auto_ui::trans::Templates;
use auto_lang::scope::Meta;
use auto_lang::ast::Name;
use std::rc::Rc;
 
fn gen_example(example: &str) {
    let code = std::fs::read_to_string(format!("crates/auto-ui/examples/{}.at", example)).unwrap();
    let universe = shared(Universe::new());
    // TODO: import real theme to scope
    universe.borrow_mut().define("theme", Rc::new(Meta::Ref(Name::new("theme"))));
    let mut trans = GpuiTrans::new(universe.clone());
    let mut out = Vec::new();
    let ast = parse_with_scope(&code, universe.clone()).unwrap();
    trans.trans(ast, &mut out).unwrap();
    println!("{}", String::from_utf8(out).unwrap());
}

fn main() {
    let examples = vec![
        "hello",
        "login",
    ];

    let story_template = Templates::story().unwrap();
    println!("{}", story_template);

    for example in examples {
        println!("Generating Example: {}", example);
        gen_example(example);
    }
}
