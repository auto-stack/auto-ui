use auto_gen::*;
use auto_val::{Obj, AutoPath, AutoStr};
use auto_atom::Atom;
use auto_lang::config::AutoConfig;

fn main() {
    // 1. read auto code
    let code = AutoStr::from(std::fs::read_to_string("crates/auto-ui/examples/hello.at").unwrap());
    // 2. Use auto-lang parser to parse the code

    println!("{}", code);

    return;

    // 2. convert auto type and main fn to atom
    let mut data = AutoConfig::from_file(AutoPath::new("crates/auto-ui/examples/hello.at").path(), &Obj::new()).unwrap();
    data.root.props.set("name", "story");
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