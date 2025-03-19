use auto_gen::*;
use auto_val::{Obj, AutoPath, AutoStr, Value};
use auto_atom::Atom;
use auto_lang::config::AutoConfig;
use auto_lang::parser::Parser;
use auto_lang::scope::Meta;
use auto_ui::trans::WidgetInfo;
use auto_lang::ast::Type;
use auto_lang::AutoResult;
use auto_lang::universe::Universe;
use std::rc::Rc;
use std::cell::RefCell;

#[allow(dead_code)]
fn gen_from_data() {
    let data = AutoConfig::from_file(AutoPath::new("crates/auto-ui/examples/hello_data.at").path(), &Obj::new()).unwrap();
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

fn main() {
    gen_from_data();
}
